#                 __
#    ____ _____  |  | _____
#   /    \\__  \ |  | \__  \
#  |   |  \/ __ \|  |__/ __ \_
#  |___|  (____  /____(____  /
#       \/     \/          \/
#
# Copyright (C) 2021, 2022 Blake Lee
#
# This file is part of nala
#
# nala is program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# nala is program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with nala.  If not, see <https://www.gnu.org/licenses/>.
"""Nala dpkg module."""
from __future__ import annotations

import errno
import fcntl
import os
import pty
import re
import signal
import struct
import sys
import termios
from time import sleep
from traceback import format_exception
from types import FrameType
from typing import Callable, Match, TextIO

import apt_pkg
from apt.progress import base, text
from pexpect.fdpexpect import fdspawn
from pexpect.utils import poll_ignore_interrupts
from ptyprocess.ptyprocess import _setwinsize

from nala import _, color, config
from nala.constants import (
	CONF_ANSWERS,
	CONF_MESSAGE,
	DPKG_ERRORS,
	DPKG_STATUS,
	ERROR_PREFIX,
	HANDLER,
	NOTICES,
	SPAM,
	WARNING_PREFIX,
)
from nala.options import arguments
from nala.rich import (
	Group,
	Live,
	Panel,
	RenderableType,
	Table,
	TaskID,
	Thread,
	ascii_replace,
	dpkg_progress,
	from_ansi,
	spinner,
)
from nala.utils import dprint, eprint, term

VERSION_PATTERN = re.compile(r"\(.*?\)")
PARENTHESIS_PATTERN = re.compile(r"[()]")

notice: list[str] = []
pkgnames: set[str] = set()
unpacked: set[str] = set()
dpkg_error: list[str] = []
update_error: list[str] = []

REMOVING = "Removing"
UNPACKING = "Unpacking"
SETTING_UP = "Setting up"
PROCESSING = "Processing"
FETCHED = _("Fetched")
GET = "GET"

UPDATED = _("Updated:")
DOWNLOADED = _("Downloaded:")
IGNORED = _("Ignored:")
ERROR = _("Error:")
NO_CHANGE = _("No Change:")


class OpProgress(text.OpProgress):
	"""Operation progress reporting.

	This closely resembles OpTextProgress in libapt-pkg.
	"""

	# we have to use this string format or else things get buggy
	# pylint: disable=consider-using-f-string
	def update(self, percent: float | None = None) -> None:
		"""Call periodically to update the user interface."""
		base.OpProgress.update(self, percent)
		if arguments.verbose:
			if self.major_change and self.old_op:
				self._write(self.old_op)

			self._write("%s... %i%%\r" % (self.op, self.percent), False, True)
			self.old_op = self.op

	def done(self) -> None:
		"""Call once an operation has been completed."""
		base.OpProgress.done(self)
		if arguments.verbose:
			if self.old_op:
				self._write(_("%c%s... Done") % ("\r", self.old_op), True, True)
			self.old_op = ""


class UpdateProgress(text.AcquireProgress):
	"""Class for getting cache update status and printing to terminal."""

	def __init__(self, live: DpkgLive) -> None:
		"""Class for getting cache update status and printing to terminal."""
		dprint("Init UpdateProgress")
		text.AcquireProgress.__init__(self)
		self._file = sys.__stdout__
		self._signal: HANDLER = None
		self._id = 1
		self._width = 80
		self.live = live

		self.live.scroll_list.clear()

	def apt_write(self, msg: str, newline: bool = True, maximize: bool = False) -> None:
		"""Write original apt update message."""
		self._file.write("\r")
		self._file.write(msg)

		# Fill remaining stuff with whitespace
		if self._width > len(msg):
			self._file.write((self._width - len(msg)) * " ")
		elif maximize:  # Needed for OpProgress.
			self._width = max(self._width, len(msg))
		if newline:
			self._file.write("\n")
		else:
			self._file.flush()

	def _write(self, msg: str, newline: bool = True, maximize: bool = False) -> None:
		"""Write the message on the terminal, fill remaining space."""
		if (
			arguments.raw_dpkg
			or not term.console.is_terminal
			or term.console.is_dumb_terminal
		):
			self.apt_write(msg, newline, maximize)
			return

		for item in (UPDATED, DOWNLOADED, IGNORED, NO_CHANGE):
			if item in msg:
				self.table_print(msg, update_spinner=True)
				break
		else:
			# For the pulse messages we need to do some formatting
			# End of the line will look like '51.8 mB/s 2s'
			if msg.endswith("s"):
				msg = fill_pulse(msg.split())

			if FETCHED in msg:
				self.table_print(msg, fetched=True)
				return

			if ERROR in msg:
				for line in msg.splitlines():
					update_error.append(line)
					eprint(line)
				return

			spinner.text = from_ansi(msg)
			self.table_print(update_spinner=True)

	def table_print(
		self, msg: str = "", fetched: bool = False, update_spinner: bool = False
	) -> None:
		"""Update wrapper for the scroll bar."""
		if not config.SCROLL and not fetched and msg:
			print(msg)
			return

		self.live.scroll_bar(
			msg,
			update_spinner=update_spinner,
			apt_fetch=self.live.install,
			use_bar=False,
		)

	def ims_hit(self, item: apt_pkg.AcquireItemDesc) -> None:
		"""Call when an item is update (e.g. not modified on the server)."""
		base.AcquireProgress.ims_hit(self, item)
		line = _("{no_change} {info}").format(
			no_change=color(NO_CHANGE, "GREEN"), info=item.description
		)
		if item.owner.filesize:
			size = apt_pkg.size_to_str(item.owner.filesize)
			line = _("{no_change} {info} [{size}B]").format(
				no_change=color(NO_CHANGE, "GREEN"), info=item.description, size=size
			)
		self._write(line)

	def fail(self, item: apt_pkg.AcquireItemDesc) -> None:
		"""Call when an item is failed."""
		base.AcquireProgress.fail(self, item)
		if item.owner.status == item.owner.STAT_DONE:
			self._write(
				_("{ignored}   {info}").format(
					ignored=color(IGNORED, "YELLOW"), info=item.description
				)
			)
		else:
			# spaces are to make the error message consistent with other messages.
			self._write(
				_("{error} {info}\n  {error_text}").format(
					error=ERROR_PREFIX,
					info=item.description,
					error_text=item.owner.error_text,
				)
			)

	def fetch(self, item: apt_pkg.AcquireItemDesc) -> None:
		"""Call when some of the item's data is fetched."""
		base.AcquireProgress.fetch(self, item)
		# It's complete already (e.g. Hit)
		if item.owner.complete:
			return
		line = _("{updated}   {info}").format(
			updated=color(DOWNLOADED if self.live.install else UPDATED, "BLUE"),
			info=item.description,
		)
		if item.owner.filesize:
			size = apt_pkg.size_to_str(item.owner.filesize)
			line = _("{updated}   {info} [{size}B]").format(
				updated=color(DOWNLOADED if self.live.install else UPDATED, "BLUE"),
				info=item.description,
				size=size,
			)
		self._write(line)

	def _winch(self, *_args: object) -> None:
		"""Signal handler for window resize signals."""
		if hasattr(self._file, "fileno") and os.isatty(self._file.fileno()):
			buf = fcntl.ioctl(self._file, termios.TIOCGWINSZ, 8 * b" ")
			dummy, columns, dummy, dummy = struct.unpack("hhhh", buf)
			self._width = columns - 1  # 1 for the cursor

	def start(self) -> None:
		"""Start an Acquire progress.

		In this case, the function sets up a signal handler for SIGWINCH, i.e.
		window resize signals. And it also sets id to 1.
		"""
		base.AcquireProgress.start(self)
		self._signal = signal.signal(signal.SIGWINCH, self._winch)
		# Get the window size.
		self._winch()
		self._id = 1

	def final_msg(self) -> str:
		"""Print closing fetched message."""
		return color(
			_("{fetched} {size}B in {elapsed} ({speed}B/s)").format(
				fetched=FETCHED,
				size=apt_pkg.size_to_str(self.fetched_bytes),
				elapsed=apt_pkg.time_to_str(self.elapsed_time),
				speed=apt_pkg.size_to_str(self.current_cps),
			)
		)

	def stop(self) -> None:
		"""Invoke when the Acquire process stops running."""
		base.AcquireProgress.stop(self)
		# We don't want to display fetched Zero if we're in apt fetch.
		if self.fetched_bytes != 0 or not self.live.install:
			self._write(self.final_msg())
		# Delete the signal again.
		signal.signal(signal.SIGWINCH, self._signal)


# We don't call super init because it opens some File Descriptors we don't need
# There is no functionality we miss out on by doing a super init
# pylint: disable=too-many-instance-attributes, too-many-public-methods
class InstallProgress(base.InstallProgress):
	"""Class for getting dpkg status and printing to terminal."""

	def __init__(
		self, dpkg_log: TextIO, term_log: TextIO, live: DpkgLive, task: TaskID
	) -> None:
		"""Class for getting dpkg status and printing to terminal."""
		dprint("Init InstallProgress")
		base.InstallProgress.__init__(self)
		self.task = task
		self._dpkg_log = dpkg_log
		self._term_log = term_log
		self.live = live
		self.raw = False
		self.bug_list = False
		self.last_line = b""
		self.child: AptExpect
		self.child_fd: int
		self.child_pid: int
		self.line_fix: list[bytes] = []
		# Setting environment to xterm seems to work fine for linux terminal
		# I don't think we will be supporting much more this this, at least for now
		if not term.is_xterm() and not arguments.raw_dpkg:
			os.environ["TERM"] = "xterm"

	def finish_update(self) -> None:
		"""Call when update has finished."""
		if not arguments.raw_dpkg:
			dpkg_progress.advance(self.task)
			self.live.scroll_bar()

	def run_install(self, apt: apt_pkg.PackageManager | list[str]) -> int:
		"""Install using the `PackageManager` object `obj`.

		returns the result of calling `obj.do_install()`
		"""
		dprint("Starting InstallProgress.run")
		pid, self.child_fd = fork()
		if pid == 0:
			try:
				# PEP-446 implemented in Python 3.4 made all descriptors
				# CLOEXEC, but we need to be able to pass writefd to dpkg
				# when we spawn it
				os.set_inheritable(self.writefd, True)
				if isinstance(apt, list):
					# nosec because this isn't really a security issue. We're just running dpkg
					os._exit(
						os.spawnlp(  # nosec
							os.P_WAIT,
							"dpkg",
							"dpkg",
							"--status-fd",
							str(self.write_stream.fileno()),
							"-i",
							*apt,
						)
					)
				# We ignore this with mypy because the attr is there
				os._exit(apt.do_install(self.write_stream.fileno()))  # type: ignore[attr-defined]
			# We need to catch every exception here.
			# If we don't the code continues in the child,
			# And bugs will be very confusing
			except Exception:  # pylint: disable=broad-except
				exception = format_exception(*sys.exc_info())
				self.dpkg_log(f"{exception}\n")
				os._exit(1)

		dprint("Dpkg Forked")
		self.child_pid = pid
		if arguments.raw_dpkg:
			return os.WEXITSTATUS(self.wait_child())
		# We use fdspawn from pexpect to interact with our dpkg pty
		# But we also subclass it to give it the interact method and setwindow
		self.child = AptExpect(self.child_fd, timeout=None)

		signal.signal(signal.SIGWINCH, self.sigwinch_passthrough)
		self.child.interact(self.pre_filter)
		return os.WEXITSTATUS(self.wait_child())

	def sigwinch_passthrough(
		self, _sig_dummy: int, _data_dummy: FrameType | None
	) -> None:
		"""Pass through sigwinch signals to dpkg."""
		buffer = struct.pack("HHHH", 0, 0, 0, 0)
		term_size = struct.unpack(
			"hhhh", fcntl.ioctl(term.STDIN, termios.TIOCGWINSZ, buffer)
		)
		if self.child.isalive():
			_setwinsize(self.child_fd, term_size[0], term_size[1])

	def conf_check(self, rawline: bytes) -> None:
		"""Check if we get a conf prompt."""
		if CONF_MESSAGE in rawline:
			self.raw_init()
		if b"Parsing Found/Fixed information... Done" in rawline and b"bugs" in rawline:
			self.bug_list = True
			self.raw_init()

	def conf_end(self, rawline: bytes) -> bool:
		"""Check to see if the conf prompt is over."""
		if self.bug_list:
			return rawline == term.CRLF and (
				b"[Y/n/?/...]" in self.last_line or self.last_line in (b"y", b"Y")
			)
		return rawline == term.CRLF and (
			CONF_MESSAGE in self.last_line or self.last_line in CONF_ANSWERS
		)

	def dpkg_log(self, msg: str) -> None:
		"""Write to dpkg-debug.log and flush."""
		self._dpkg_log.write(msg)
		self._dpkg_log.flush()

	def term_log(self, msg: bytes) -> None:
		"""Write to term.log and flush."""
		self._term_log.write(f"{msg.decode('utf-8').strip()}\n")
		self._term_log.flush()

	def dpkg_status(self, data: bytes) -> bool:
		"""Handle any status messages."""
		for status in DPKG_STATUS:
			if status in data:
				if status in (
					b"[Working]",
					b"[Connecting",
					b"[Waiting for headers]",
					b"[Connected to",
				):
					return True
				statuses = data.split(b"\r")
				if len(statuses) > 2:
					self.dpkg_log(f"Status_Split = {repr(statuses)}\n")
				for msg in statuses:
					if msg != b"":
						spinner.text = from_ansi(color(msg.decode().strip()))
						self.live.scroll_bar(update_spinner=True)
				self.dpkg_log(term.LF.decode())
				return True
		return False

	def apt_diff_pulse(self, data: bytes) -> bool:
		"""Handle pulse messages from apt-listdifferences."""
		if data.startswith(b"\r") and data.endswith(b"s"):
			spinner.text = from_ansi(color(fill_pulse(data.decode().split())))
			self.live.scroll_bar(update_spinner=True)
			return True
		return False

	def apt_differences(self, data: bytes) -> bool:
		"""Handle messages from apt-listdifferences."""
		if not data.strip().endswith((b"%", b"%]")):
			return False
		for line in data.splitlines():
			if any(item in line.decode() for item in SPAM) or not line:
				continue
			if b"Get" in line:
				self.dpkg_log(f"Get = [{repr(line)}]\n")
				self.format_dpkg_output(line)
				continue
			pulse = [msg for msg in line.decode().split() if msg]
			self.dpkg_log(f"Difference = [{pulse}]\n")
			spinner.text = from_ansi(color(" ".join(pulse)))
			self.live.scroll_bar(update_spinner=True)
		return True

	def read_status(self) -> None:
		"""Read the status fd and send it to update progress bar."""
		try:
			status = self.status_stream.read(1024)
		except OSError as err:
			# Resource temporarily unavailable is ignored
			if err.errno not in (errno.EAGAIN, errno.EWOULDBLOCK):
				print(err.strerror)
			return
		for line in status.splitlines():
			self.update_progress_bar(line)

	def update_progress_bar(self, line: str) -> None:
		"""Update the interface."""
		pkgname = status = status_str = _percent = base_status = ""

		if line.startswith("pm"):
			try:
				(status, pkgname, _percent, status_str) = line.split(":", 3)
			except ValueError:
				# Silently ignore lines that can't be parsed
				return
		elif line.startswith("status"):
			try:
				(base_status, pkgname, status, status_str) = line.split(":", 3)
			except ValueError:
				(base_status, pkgname, status) = line.split(":", 2)

		# Always strip the status message
		pkgname = pkgname.strip()
		status_str = status_str.strip()
		status = status.strip()

		# This is the main branch for apt installs
		if status == "pmstatus":
			dprint(f"apt: {pkgname} {status_str}")
			if status_str.startswith(("Unpacking", "Removing")):
				unpacked.add(pkgname)
				self.advance_progress()
			# Either condition can satisfy this mark provided the package hasn't been advanced
			elif (
				status_str.startswith(("Installed", "Configuring"))
				and pkgname not in pkgnames
				and pkgname in unpacked
			):
				pkgnames.add(pkgname)
				self.advance_progress()
		# This branch only happens for local .deb installs.
		elif base_status == "status":
			# Sometimes unpacked is notified twice for one package
			# We check against out set to make sure not to over shoot progress
			if status == "unpacked" and pkgname not in pkgnames:
				self.advance_progress()
				pkgnames.add(pkgname)
			# Sometimes packages are notified as installed
			# But we only care for ones that have been unpacked
			if status == "installed" and pkgname in pkgnames:
				self.advance_progress()
			dprint(f"dpkg: {pkgname} {status}")

	def pre_filter(self, data: bytes) -> None:
		"""Filter data from interact."""
		self.read_status()
		self.conf_check(data)

		# This is a work around for a hang in non-interactive mode
		# https://github.com/liske/needrestart/issues/129
		if (
			os.environ.get("DEBIAN_FRONTEND") == "noninteractive"
			and b"so you should consider rebooting. [Return]" in data
		):
			os.write(self.child_fd, term.CRLF)

		# Save Term for debconf and Bracked Paste for the start of the shell
		if term.SAVE_TERM in data or term.ENABLE_BRACKETED_PASTE in data:
			self.raw_init()

		self.dpkg_log(f"Raw = {self.raw}: [{repr(data)}]\n")

		if not self.raw:
			if self.dpkg_status(data):
				return
			if self.apt_diff_pulse(data) or self.apt_differences(data):
				return

			if not data.endswith(term.CRLF):
				self.line_fix.append(data)
				return

			if self.line_fix:
				self.dpkg_log(f"line_fix = {repr(self.line_fix)}\n")
				data = b"".join(self.line_fix) + data
				self.line_fix.clear()

			if data.count(b"\r\n") > 1:
				self.split_data(data)
				return
		self.format_dpkg_output(data)

	def split_data(self, data: bytes) -> None:
		"""Split data into clean single lines to format."""
		data_split = data.split(b"\r\n")
		error = data_split[0].decode() in dpkg_error
		self.dpkg_log(f"Data_Split = {repr(data_split)}\n")
		for line in data_split:
			for new_line in line.split(b"\r"):
				if new_line:
					check_error(data, new_line.decode(), error)
					self.format_dpkg_output(new_line)

	def format_dpkg_output(self, rawline: bytes) -> None:
		"""Facilitate what needs to happen to dpkg output."""
		# If we made it here that means we're okay to start a new line in the log
		self.dpkg_log(term.LF.decode())

		if self.raw:
			self.rawline_handler(rawline)
			return
		self.line_handler(rawline)

	def line_handler(self, rawline: bytes) -> None:
		"""Handle text operations for not a rawline."""
		line = ascii_replace(rawline.decode().strip())

		if check_line_spam(line, rawline, self.last_line):
			return

		# Percent is for apt-listdifferences, b'99% [6  1988 kB]'
		if line == "" or "% [" in line:
			return

		self.term_log(rawline)

		# Main format section for making things pretty
		msg = msg_formatter(line)
		# If verbose we just send it. No bars
		if not config.SCROLL:
			print(msg)
			self.live.scroll_bar()
		elif "Fetched:" in msg:
			# This is some magic for apt-listdifferences to put
			# the fetched message in the spinner since it gets spammy
			spinner.text = from_ansi(color(" ".join(line.split()[1:])))
			self.live.scroll_bar(msg, update_spinner=True)
		else:
			self.live.scroll_bar(msg)
		sys.__stdout__.flush()
		self.set_last_line(rawline)

	def rawline_handler(self, rawline: bytes) -> None:
		"""Handle text operations for rawline."""
		term.write(rawline)
		# Once we write we can check if we need to pop out of raw mode
		if term.RESTORE_TERM in rawline or self.conf_end(rawline):
			self.raw = False
			self.bug_list = False
			term.restore_mode()
			self.live.start()
		self.set_last_line(rawline)

	def set_last_line(self, rawline: bytes) -> None:
		"""Set the current line to last line if there is no backspace."""
		# When at the conf prompt if you press Y, then backspace, then hit enter
		# Things get really buggy so instead we check for a backspace
		if term.BACKSPACE not in rawline:
			self.last_line = rawline

	def advance_progress(self) -> None:
		"""Advance the dpkg progress bar."""
		dpkg_progress.advance(self.task)
		if not config.SCROLL:
			self.live.update(
				Panel.fit(
					dpkg_progress.get_renderable(),
					border_style="bold green",
					padding=(0, 0),
				),
				refresh=True,
			)

	def raw_init(self) -> None:
		"""Initialize raw terminal output."""
		if self.raw:
			return
		self.live.raw_init()
		term.set_raw()
		self.raw = True


def check_line_spam(line: str, rawline: bytes, last_line: bytes) -> bool:
	"""Check for, and handle, notices and spam."""
	for message in NOTICES:
		if message in rawline and line not in notice:
			notice.append(line)
			return False
	if b"but it can still be activated by:" in last_line:
		notice.append(f"  {line}")
		return False

	return any(item in line for item in SPAM)


def check_error(data: bytes, line: str, error_in_list: bool = False) -> None:
	"""Check dpkg errors and store them if we need too."""
	# Check so we don't duplicate error messages
	if error_in_list:
		return
	for error in DPKG_ERRORS:
		# Make sure that the error is not spam
		if error in data and all(item not in line for item in SPAM):
			dpkg_error.append(line)


def paren_color(match: Match[str]) -> str:
	"""Color parenthesis."""
	return color("(") if match.group(0) == "(" else color(")")


def lines(line: str, zword: str, msg_color: str) -> str:
	"""Color and space our line."""
	space = " "
	if zword == REMOVING:
		space *= 3
	elif zword == UNPACKING:
		space *= 2
	return line.replace(zword, color(f"{zword}:{space}", msg_color))


def format_version(match: list[str], line: str) -> str:
	"""Format version numbers."""
	for ver in match:
		version = ver[1:-1]
		if version[0].isdigit():
			new_ver = ver.replace(version, color(version, "BLUE"))
			new_ver = re.sub(PARENTHESIS_PATTERN, paren_color, new_ver)
			line = line.replace(ver, new_ver)
	return line


def fill_pulse(pulse: list[str]) -> str:
	"""Fill the pulse message."""
	last = len(pulse) - 1
	fill = sum(len(line) for line in pulse) + last
	# Set fill width to fit inside the rich panel
	fill = (term.columns - fill) - 5 if arguments.verbose else (term.columns - fill) - 7
	# Minus 2 more to account for ascii simpleDots on the spinner
	if term.console.options.ascii_only:
		fill -= 2
	# Make sure we insert the filler in the right spot
	# In case of extra 1 min as shown below.
	# ['2407', 'kB/s', '30s']
	# ['895', 'kB/s', '1min', '18s']
	index = last - 2
	if "/s" in pulse[index]:
		index = last - 3

	pulse.insert(index, " " * fill)
	return " ".join(pulse)


def msg_formatter(line: str) -> str:
	"""Format dpkg output."""
	if line.endswith("..."):
		line = line.replace("...", "")

	if line.startswith(REMOVING):
		line = lines(line, REMOVING, "RED")
	elif line.startswith(UNPACKING):
		line = lines(line, UNPACKING, "GREEN")
	elif line.startswith(SETTING_UP):
		line = lines(line, SETTING_UP, "GREEN")
	elif line.startswith(PROCESSING):
		line = lines(line, PROCESSING, "GREEN")
	elif line.startswith(GET):
		line = f"{color(f'{FETCHED}:', 'BLUE')} {' '.join(line.split()[1:])}"

	if match := re.findall(VERSION_PATTERN, line):
		return format_version(match, line)
	return line


class DpkgLive(Live):
	"""Subclass for dpkg live display."""

	def __init__(self, install: bool = True) -> None:
		"""Subclass for dpkg live display."""
		super().__init__(refresh_per_second=10)
		self.install = install
		self.scroll_list: list[str] = []
		self.scroll_config = (False, False, True)
		self.used_scroll: bool = False

	def __enter__(self) -> DpkgLive:
		"""Start the live display."""
		self.start(refresh=self._renderable is not None)
		return self

	def scroll_bar(  # pylint: disable=too-many-arguments
		self,
		msg: str = "",
		apt_fetch: bool = False,
		update_spinner: bool = False,
		use_bar: bool = True,
		rerender: bool = False,
	) -> None:
		"""Print msg to our scroll bar live display."""
		if rerender:
			if not self.used_scroll:
				return
			apt_fetch, update_spinner, use_bar = self.scroll_config
		else:
			self.used_scroll = True
			self.scroll_config = (apt_fetch, update_spinner, use_bar)

		if msg:
			self.scroll_list.append(msg)
		self.slice_list()

		table = Table.grid()
		table.add_column(no_wrap=True, width=term.columns, overflow=term.overflow)

		for item in self.scroll_list:
			table.add_row(from_ansi(item))

		if use_bar or update_spinner:
			table.add_row(
				Panel(
					self.get_group(update_spinner, use_bar),
					padding=(0, 0),
					border_style="bold blue" if config.SCROLL else "bold green",
				)
			)

		# We don't need to build the extra panel if we're not scrolling
		if not config.SCROLL:
			self.update(table, refresh=True)
			return

		self.update(
			Panel(
				table,
				title=self.get_title(self.install, apt_fetch),
				title_align="left",
				padding=(0, 0),
				border_style="bold green",
			),
			refresh=True,
		)

	@staticmethod
	def get_title(install: bool, apt_fetch: bool) -> str:
		"""Get the title for our panel."""
		msg = "[bold default]"
		if arguments.command and install and not apt_fetch:
			if arguments.command in ("remove", "purge"):
				msg += _("Removing Packages")
			elif arguments.command == "upgrade":
				msg += _("Updating Packages")
			elif arguments.command == "install":
				msg += _("Installing Packages")
			elif arguments.command == "history":
				title = (
					_("History Undo") if arguments.mode == "undo" else _("History Redo")
				)
				msg += f"{title} {arguments.id}"
			return msg
		if install and apt_fetch:
			return msg + _("Fetching Missed Packages")
		if not arguments.command and arguments.fix_broken:
			return msg + _("Fixing Broken Packages")
		return msg + _("Updating Package List")

	@staticmethod
	def get_group(update_spinner: bool, use_bar: bool) -> RenderableType:
		"""Get the group for our panel."""
		if spinner.text.plain:  # type: ignore[union-attr]
			if update_spinner and use_bar:
				return Group(
					spinner,
					dpkg_progress.get_renderable(),
				)
			if update_spinner:
				return spinner
		return dpkg_progress.get_renderable()

	def slice_list(self) -> None:
		"""Set scroll bar to take up only 1/2 of the screen."""
		scroll_lines = term.lines // 2
		size = len(self.scroll_list)
		if size > scroll_lines and size > 10:
			total = size - max(scroll_lines, 10)
			self.scroll_list = self.scroll_list[total:]

	def raw_init(self) -> None:
		"""Set up the live display to be stopped."""
		# Stop the live display from Auto Refreshing
		if self._refresh_thread:
			self._refresh_thread.stop()
			self._refresh_thread = None

		# We update the live display to blank before stopping it
		self.update("", refresh=True)
		self.stop()

	def start(self, refresh: bool = False) -> None:
		"""Start live rendering display.

		Args:
		----
		        refresh (bool, optional): Also refresh. Defaults to False.

		"""
		with self._lock:
			if self._started:
				return
			self.console.set_live(self)
			self._started = True
			if self._screen:
				self._alt_screen = self.console.set_alt_screen(True)
			self.console.show_cursor(False)
			self._enable_redirect_io()
			self.console.push_render_hook(self)
			if refresh:
				self.refresh()
			if self.auto_refresh:
				self._refresh_thread = Thread(self, self.refresh_per_second)
				self._refresh_thread.start()


def fork() -> tuple[int, int]:
	"""Fork pty or regular."""
	return (os.fork(), 0) if arguments.raw_dpkg else pty.fork()


class AptExpect(fdspawn):  # type: ignore[misc]
	"""Subclass of fdspawn to add the interact method."""

	def interact(self, output_filter: Callable[[bytes], None]) -> None:
		"""Hacked up interact method.

		Because pexpect doesn't want to have one for fdspawn.

		This gives control of the child process to the interactive user (the
		human at the keyboard). Keystrokes are sent to the child process, and
		the stdout and stderr output of the child process is printed. This
		simply echos the child stdout and child stderr to the real stdout and
		it echos the real stdin to the child stdin.
		"""
		# Flush the buffer.
		self.write_to_stdout(self.buffer)
		self.stdout.flush()
		self._buffer = self.buffer_type()

		_setwinsize(self.child_fd, term.lines, term.columns)
		# _setecho(self.child_fd, False)

		self.interact_copy(output_filter)

	def interact_copy(self, output_filter: Callable[[bytes], None]) -> None:
		"""Interact with the pty."""
		while self.isalive():
			try:
				ready = poll_ignore_interrupts([self.child_fd, term.STDIN])
				if self.child_fd in ready and not self._read(output_filter):
					break
				if term.STDIN in ready:
					self._write()
			except KeyboardInterrupt:
				term.write(term.CURSER_UP + term.CLEAR_LINE)
				eprint(
					_("{warning} Quitting now could break your system!").format(
						warning=WARNING_PREFIX
					)
				)
				eprint(color(_("Ctrl+C twice quickly will exit..."), "RED"))
				sleep(0.5)

	def _read(self, output_filter: Callable[[bytes], None]) -> bool:
		"""Read data from the pty and send it for formatting."""
		try:
			data = os.read(self.child_fd, 1000)
		except OSError as err:
			if err.args[0] == errno.EIO:
				# Linux-style EOF
				return False
			raise
		if data == b"":
			# BSD-style EOF
			return False
		output_filter(data)
		return True

	def _write(self) -> None:
		"""Write user inputs into the pty."""
		data = os.read(term.STDIN, 1000)
		while data != b"" and self.isalive():
			split = os.write(self.child_fd, data)
			data = data[split:]
