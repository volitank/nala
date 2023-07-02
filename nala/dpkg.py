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
# nala is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# nala is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with nala.  If not, see <https://www.gnu.org/licenses/>.
"""Nala dpkg module."""
from __future__ import annotations

import contextlib
import errno
import fcntl
import os
import pty
import re
import signal
import struct
import sys
import termios
from time import sleep, time
from traceback import format_exception
from types import FrameType
from typing import Callable, Match, TextIO

import apt_pkg
from apt.progress import base, text
from pexpect.fdpexpect import fdspawn
from pexpect.utils import poll_ignore_interrupts
from ptyprocess.ptyprocess import _setwinsize

from nala import _, color
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
	ELLIPSIS,
	OVERFLOW,
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
	to_str,
)
from nala.utils import dprint, eprint, term, unit_str

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
GET = "GET"

UPDATED = _("Updated:")
DOWNLOADED = _("Downloaded:")
IGNORED = _("Ignored:")
NO_CHANGE = _("No Change:")

# NOTE: Spacing of following status messages
# NOTE: is to allow the urls to be properly aligned
# NOTE: Especially if your status would come after the package
# NOTE: You do not have to follow this scheme
# NOTE: but do note that the headers will be colored regardless
# NOTE: No Change: http://deb.volian.org/volian scar InRelease
# NOTE: Ignored:   http://deb.volian.org/volian scar InRelease
# NOTE: Updated:   http://deb.volian.org/volian scar InRelease
NO_CHANGE_MSG = _("{no_change} {info}")
NO_CHANGE_SIZE_MSG = _("{no_change} {info} [{size}]")
IGNORED_MSG = _("{ignored}   {info}")
UPDATE_MSG = _("{updated}   {info}")
UPDATE_SIZE_MSG = _("{updated}   {info} [{size}]")

REMOVING_HEAD = color(_("Removing:"), "RED")
UNPACKING_HEAD = color(_("Unpacking:"), "GREEN")
SETTING_UP_HEAD = color(_("Setting up:"), "GREEN")
PROCESSING_HEAD = color(_("Processing:"), "GREEN")

# NOTE: Spacing of following status messages
# NOTE: is to allow dpkg messages to be properly aligned
# NOTE: Especially if your status would come after the package
# NOTE: You do not have to follow this scheme
# NOTE: but do note that the headers will be colored regardless
# NOTE: Unpacking:   neofetch (7.1.0-3)
# NOTE: Setting up:  neofetch (7.1.0-3)
# NOTE: Removing:    neofetch (7.1.0-3)
# NOTE: Processing:  triggers for man-db (2.10.2-1)
# NOTE: You can change the headers and positions as you would like,
# NOTE: but do note that the headers will be colored regardless
SETTING_UP_MSG = _("{setting_up} {dpkg_msg}")
PROCESSING_MSG = _("{processing} {dpkg_msg}")
UNPACKING_MSG = _("{unpacking}  {dpkg_msg}")
# NOTE: That's the end of alignment spacing
REMOVING_MSG = _("{removing}   {dpkg_msg}")

# NOTE: This translation is separate from the one below
# NOTE: Because we do a check specifically on this string
FETCHED = _("Fetched")
# NOTE: Fetched 81.0 MB in 6s (1448 kB/s)
FETCHED_MSG = _("{fetched} {size} in {elapsed} ({speed}/s)")


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
		self.elapsed = 0.0

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
		if arguments.raw_dpkg or not term.can_format():
			self.apt_write(msg, newline, maximize)
			return

		for item in (UPDATED, DOWNLOADED, IGNORED, NO_CHANGE):
			if item in msg:
				self.table_print(msg, update_spinner=True)
				break
		else:
			if FETCHED in msg:
				self.table_print(msg, fetched=True)
				return

			if ERROR_PREFIX in msg:
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
		if not arguments.scroll and not fetched and msg:
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
		line = NO_CHANGE_MSG.format(
			no_change=color(NO_CHANGE, "GREEN"), info=item.description
		)
		if item.owner.filesize:
			line = NO_CHANGE_SIZE_MSG.format(
				no_change=color(NO_CHANGE, "GREEN"),
				info=item.description,
				size=unit_str(item.owner.filesize).strip(),
			)
		self._write(line)

	def fail(self, item: apt_pkg.AcquireItemDesc) -> None:
		"""Call when an item is failed."""
		base.AcquireProgress.fail(self, item)
		if item.owner.status == item.owner.STAT_DONE:
			self._write(
				IGNORED_MSG.format(
					ignored=color(IGNORED, "YELLOW"), info=item.description
				)
			)
		else:
			# This doesn't need to be translated. Just an error dump
			self._write(f"{ERROR_PREFIX} {item.description}\n  {item.owner.error_text}")

	def fetch(self, item: apt_pkg.AcquireItemDesc) -> None:
		"""Call when some of the item's data is fetched."""
		base.AcquireProgress.fetch(self, item)
		# It's complete already (e.g. Hit)
		if item.owner.complete:
			return
		line = UPDATE_MSG.format(
			updated=color(DOWNLOADED if self.live.install else UPDATED, "BLUE"),
			info=item.description,
		)
		if item.owner.filesize:
			line = UPDATE_SIZE_MSG.format(
				updated=color(DOWNLOADED if self.live.install else UPDATED, "BLUE"),
				info=item.description,
				size=unit_str(item.owner.filesize).strip(),
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
		self.elapsed = time()
		self._signal = signal.signal(signal.SIGWINCH, self._winch)
		# Get the window size.
		self._winch()
		self._id = 1

	def final_msg(self) -> str:
		"""Print closing fetched message."""
		elapsed_time = int(time() - self.elapsed)
		if elapsed_time == 0:
			fetched_speed = self.current_cps
		else:
			fetched_speed = int(self.fetched_bytes) / elapsed_time
		return color(
			FETCHED_MSG.format(
				fetched=FETCHED,
				size=unit_str(int(self.fetched_bytes)).strip(),
				elapsed=apt_pkg.time_to_str(elapsed_time),
				speed=to_str(int(fetched_speed), 1000).strip(),
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

	def pulse(  # pylint: disable=too-many-branches
		self, owner: apt_pkg.Acquire
	) -> bool:
		"""Periodically invoked while the Acquire process is underway."""
		base.AcquireProgress.pulse(self, owner)
		# only show progress on a tty to not clutter log files etc
		if hasattr(self._file, "fileno") and not os.isatty(self._file.fileno()):
			return True

		# calculate progress
		percent = ((self.current_bytes + self.current_items) * 100.0) / (
			self.total_bytes + self.total_items
		)

		shown = False
		tval = f"{percent:.0f}%"
		end = ""
		if self.current_cps:
			eta = int((self.total_bytes - self.current_bytes) / self.current_cps)
			end = f" {unit_str(int(self.current_cps)).strip()}/s {apt_pkg.time_to_str(eta)}"

		for worker in owner.workers:
			val = ""
			if not worker.current_item:
				if worker.status:
					val = f" [{worker.status}]"
					if len(tval) + len(val) + len(end) >= self._width - 5:
						break
					tval += val
					shown = True
				continue
			shown = True

			if worker.current_item.owner.id:
				val += (
					f" [{worker.current_item.owner.id} {worker.current_item.shortdesc}"
				)
			else:
				val += f" [{worker.current_item.description}"
			if worker.current_item.owner.active_subprocess:
				val += f" {worker.current_item.owner.active_subprocess}"

			val += f" {unit_str(worker.current_size).strip()}"

			# Add the total size and percent
			if worker.total_size and not worker.current_item.owner.complete:
				val += (
					f"/{unit_str(worker.total_size).strip()}"
					f" {(worker.current_size * 100.0) / worker.total_size:.0f}%"
				)

			val += "]"

			if len(tval) + len(val) + len(end) >= self._width - 5:
				# Display as many items as screen width
				break
			tval += val

		if not shown:
			tval += _(" [Working]")

		if self.current_cps:
			tval += (self._width - len(end) - len(tval) - 5) * " " + end

		self._write(tval, False)
		return True


# pylint: disable=too-many-instance-attributes, too-many-public-methods, too-many-arguments, too-many-lines
class InstallProgress(base.InstallProgress):
	"""Class for getting dpkg status and printing to terminal."""

	def __init__(
		self,
		dpkg_log: TextIO,
		term_log: TextIO,
		live: DpkgLive,
		task: TaskID,
		config_purge: tuple[str, ...],
	) -> None:
		"""Class for getting dpkg status and printing to terminal."""
		dprint("Init InstallProgress")
		base.InstallProgress.__init__(self)
		self.task = task
		self._dpkg_log = dpkg_log
		self._term_log = term_log
		self.live = live
		self.config_purge = config_purge
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
		dprint("Forking")
		pid, self.child_fd = fork()
		if pid == 0:
			try:
				# PEP-446 implemented in Python 3.4 made all descriptors
				# CLOEXEC, but we need to be able to pass writefd to dpkg
				# when we spawn it
				os.set_inheritable(self.writefd, True)
				if not isinstance(apt, apt_pkg.PackageManager):
					# pylint: disable=subprocess-run-check
					self.dpkg_log("Command Execution:\n")
					self.dpkg_log(f"Command = {apt}\n\n")
					os._exit(
						os.spawnlp(  # nosec
							os.P_WAIT,
							"dpkg",
							"dpkg",
							"--status-fd",
							f"{self.write_stream.fileno()}",
							"-i",
							*apt,
						)
					)
				# We ignore this with mypy because the attr is there
				self.dpkg_log("Apt Do Install\n\n")
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
			with contextlib.suppress(ValueError):
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
			and b"[Return]" in data
		):
			os.write(self.child_fd, term.CRLF)

		# Save Term and Alt Screen for debconf and Bracked Paste for the start of the shell
		if (
			term.SAVE_TERM in data
			or term.ENABLE_BRACKETED_PASTE in data
			or term.ENABLE_ALT_SCREEN in data
		):
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

		if (
			self.config_purge
			and "Purging configuration files" in line
			and any(pkg in line for pkg in self.config_purge)
		):
			self.advance_progress()

		self.term_log(rawline)

		# Main format section for making things pretty
		msg = msg_formatter(line)
		# If verbose we just send it. No bars
		if not arguments.scroll:
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
		if (
			term.RESTORE_TERM in rawline
			or term.DISABLE_ALT_SCREEN in rawline
			or self.conf_end(rawline)
			# Fix for Dialog Debconf Frontend https://gitlab.com/volian/nala/-/issues/211
		) and term.ENABLE_ALT_SCREEN not in rawline:
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
		if not arguments.scroll:
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


def line_replace(line: str, header: str) -> str:
	"""Replace wrapper for removing header."""
	return line.replace(header, "").strip()


def format_version(match: list[str], line: str) -> str:
	"""Format version numbers."""
	for ver in match:
		version = ver[1:-1]
		if version and version[0].isdigit():
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
		line = REMOVING_MSG.format(
			removing=REMOVING_HEAD, dpkg_msg=line_replace(line, REMOVING)
		)
	elif line.startswith(UNPACKING):
		line = UNPACKING_MSG.format(
			unpacking=UNPACKING_HEAD, dpkg_msg=line_replace(line, UNPACKING)
		)
	elif line.startswith(SETTING_UP):
		line = SETTING_UP_MSG.format(
			setting_up=SETTING_UP_HEAD, dpkg_msg=line_replace(line, SETTING_UP)
		)
	elif line.startswith(PROCESSING):
		line = PROCESSING_MSG.format(
			processing=PROCESSING_HEAD, dpkg_msg=line_replace(line, PROCESSING)
		)
	elif line.startswith(GET):
		line = f"{color(f'{FETCHED}:', 'BLUE')} {' '.join(line.split()[1:])}"

	if match := re.findall(VERSION_PATTERN, line):
		return format_version(match, line)
	return line


class DpkgLive(Live):
	"""Subclass for dpkg live display."""

	def __init__(self, install: bool = True) -> None:
		"""Subclass for dpkg live display."""
		super().__init__(refresh_per_second=4)
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
		table.add_column(no_wrap=True, width=term.columns, overflow=OVERFLOW)

		for item in self.scroll_list:
			table.add_row(from_ansi(item))

		if use_bar or update_spinner:
			table.add_row(
				Panel(
					self.get_group(update_spinner, use_bar),
					padding=(0, 0),
					border_style="bold blue" if arguments.scroll else "bold green",
				)
			)

		# We don't need to build the extra panel if we're not scrolling
		if not arguments.scroll:
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
			if arguments.command in ("remove", "purge", "autoremove", "autopurge"):
				if arguments.is_purge():
					msg += _("Purging Packages")
				else:
					msg += _("Removing Packages")
			elif arguments.command == "upgrade":
				msg += _("Updating Packages")
			elif arguments.command == "install":
				msg += _("Installing Packages")
			elif arguments.command == "history":
				title = (
					_("History Undo")
					if arguments.history == "undo"
					else _("History Redo")
				)
				msg += f"{title} {arguments.history_id}"
			return msg
		if install and apt_fetch:
			return msg + _("Fetching Missed Packages")
		if not arguments.command and arguments.fix_broken:
			return msg + _("Fixing Broken Packages")
		return msg + _("Updating Package List")

	@staticmethod
	def get_group(update_spinner: bool, use_bar: bool) -> RenderableType:
		"""Get the group for our panel."""
		if spinner.text.plain and update_spinner:  # type: ignore[union-attr]
			return (
				Group(spinner, dpkg_progress.get_renderable()) if use_bar else spinner
			)

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
				eprint(color(_("Ctrl+C twice quickly will exit") + ELLIPSIS, "RED"))
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
