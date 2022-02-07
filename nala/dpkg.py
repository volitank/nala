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
from types import FrameType
from typing import Callable, Match, TextIO, cast

import apt_pkg
from apt.progress import base, text
from pexpect.fdpexpect import fdspawn
from pexpect.utils import poll_ignore_interrupts
from rich.console import Group
from rich.panel import Panel
from rich.progress import TaskID

from nala.constants import DPKG_MSG, ERROR_PREFIX, HANDLER, SPAM
from nala.options import arguments
from nala.rich import Live, Spinner, Table, Text, dpkg_progress
from nala.utils import color, term

VERSION_PATTERN = re.compile(r'\(.*?\)')
PARENTHESIS_PATTERN = re.compile(r'[()]')

spinner = Spinner('dots', text='Initializing', style="bold blue")
scroll_list: list[str] = []
notice: set[str] = set()

class OpProgress(base.OpProgress, text.TextProgress):
	"""Operation progress reporting.

	This closely resembles OpTextProgress in libapt-pkg.
	"""

	def __init__(self) -> None:
		"""Operation progress reporting."""
		text.TextProgress.__init__(self)
		base.OpProgress.__init__(self)
		self.old_op = ""

	def update(self, percent: float | None = None) -> None:
		"""Call periodically to update the user interface."""
		base.OpProgress.update(self, percent)
		if arguments.verbose:
			if self.major_change and self.old_op:
				self._write(self.old_op)
			self._write(f"{self.op}... {self.percent}%\r", False, True)
			self.old_op = self.op

	def done(self) -> None:
		"""Call once an operation has been completed."""
		base.OpProgress.done(self)
		if arguments.verbose:
			if self.old_op:
				self._write(f"\r{self.old_op}... Done", True, True)
			self.old_op = ""

class UpdateProgress(text.AcquireProgress):
	"""Class for getting cache update status and printing to terminal."""

	def __init__(self, live: Live | None = None, install: bool = False) -> None:
		"""Class for getting cache update status and printing to terminal."""
		text.AcquireProgress.__init__(self)
		self._file = sys.__stdout__
		self._signal: HANDLER = None
		self._id = 1
		self._width = 80
		self.install = install
		self.live = live or Live(auto_refresh=False)
		if arguments.debug:
			arguments.verbose=True

		spinner.text = Text('Initializing Cache')
		scroll_list.clear()

	def apt_write(self, msg: str, newline: bool = True, maximize: bool = False) -> None:
		"""Write original apt update message."""
		self._file.write("\r")
		self._file.write(msg)

		# Fill remaining stuff with whitespace
		if self._width > len(msg):
			self._file.write((self._width - len(msg)) * ' ')
		elif maximize:  # Needed for OpProgress.
			self._width = max(self._width, len(msg))
		if newline:
			self._file.write("\n")
		else:
			self._file.flush()

	def _write(self, msg: str, newline: bool = True, maximize: bool = False) -> None:
		"""Write the message on the terminal, fill remaining space."""
		if arguments.raw_dpkg:
			self.apt_write(msg, newline, maximize)
			return

		for item in ['Updated:', 'Ignored:', 'Error:', 'No Change:']:
			if item in msg:
				if arguments.verbose:
					print(msg)
					break
				scroll_bar(self,
					msg, install=self.install, fetch=self.install,
					update_spinner=True, use_bar=False
				)
				break
		else:
			# For the pulse messages we need to do some formatting
			# End of the line will look like '51.8 mB/s 2s'
			if msg.endswith('s'):
				msg = fill_pulse(msg.split())

			if 'Fetched' in msg:
				scroll_bar(self,
					msg, install=self.install,
					fetch=self.install, use_bar=False
				)
				return
			spinner.text = Text.from_ansi(msg)
			scroll_bar(self,
				install=self.install, fetch=self.install,
				update_spinner=True, use_bar=False
			)

	def ims_hit(self, item: apt_pkg.AcquireItemDesc) -> None:
		"""Call when an item is update (e.g. not modified on the server)."""
		base.AcquireProgress.ims_hit(self, item)
		self.write_update('No Change:', 'GREEN', item)

	def fail(self, item: apt_pkg.AcquireItemDesc) -> None:
		"""Call when an item is failed."""
		base.AcquireProgress.fail(self, item)
		if item.owner.status == item.owner.STAT_DONE:
			self._write(f"{color('Ignored:  ', 'YELLOW')} {item.description}")
		else:
			# spaces are to make the error message consistent with other messages.
			self._write(ERROR_PREFIX+'    '+item.description)
			self._write(f"  {item.owner.error_text}")

	def fetch(self, item: apt_pkg.AcquireItemDesc) -> None:
		"""Call when some of the item's data is fetched."""
		base.AcquireProgress.fetch(self, item)
		# It's complete already (e.g. Hit)
		if item.owner.complete:
			return
		self.write_update('Updated:  ', 'BLUE', item)

	def write_update(self, msg: str, _color: str, item: apt_pkg.AcquireItemDesc) -> None:
		"""Write the update from either hit or fetch."""
		line = f'{color(msg, _color)} {item.description}'
		if item.owner.filesize:
			size = apt_pkg.size_to_str(item.owner.filesize)
			line += f' [{size}B]'
		self._write(line)

	def _winch(self, *_args: object) -> None:
		"""Signal handler for window resize signals."""
		if hasattr(self._file, "fileno") and os.isatty(self._file.fileno()):
			buf = fcntl.ioctl(self._file, termios.TIOCGWINSZ, 8 * b' ')
			term.lines, term.columns, dummy, dummy = struct.unpack('hhhh', buf)
			self._width = term.columns - 1  # 1 for the cursor

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
		fetched = apt_pkg.size_to_str(self.fetched_bytes)
		elapsed = apt_pkg.time_to_str(self.elapsed_time)
		speed = apt_pkg.size_to_str(self.current_cps).rstrip("\n")
		return color(f"Fetched {fetched}B in {elapsed} ({speed}B/s)")

	def stop(self) -> None:
		"""Invoke when the Acquire process stops running."""
		base.AcquireProgress.stop(self)
		if self.fetched_bytes != 0:
			self._write(self.final_msg())
		# Delete the signal again.
		signal.signal(signal.SIGWINCH, self._signal)

# We don't call super init because it opens some File Descriptors we don't need
# There is no functionality we miss out on by doing a super init
# pylint: disable=too-many-instance-attributes, super-init-not-called
class InstallProgress(base.InstallProgress):
	"""Class for getting dpkg status and printing to terminal."""

	def __init__(self, dpkg_log: TextIO, live: Live, task: TaskID) -> None:
		"""Class for getting dpkg status and printing to terminal."""
		self.task = task
		self._dpkg_log = dpkg_log
		self.live = live
		self.raw = False
		self.last_line = b''
		self.child: AptExpect
		self.child_fd: int
		self.child_pid: int
		self.line_fix: list[bytes] = []
		# If we detect we're piped it's probably best to go raw.
		if not term.is_term():
			arguments.raw_dpkg = True
		# Setting environment to xterm seems to work fine for linux terminal
		# I don't think we will be supporting much more this this, at least for now
		if not term.is_xterm() and not arguments.raw_dpkg:
			os.environ["TERM"] = 'xterm'

	def finish_update(self) -> None:
		"""Call when update has finished."""
		if not arguments.raw_dpkg:
			dpkg_progress.advance(self.task)
			scroll_bar(self)

	# Exit is overridden because it closes those file descriptors we don't init with
	def __exit__(self, _type: object, value: object, traceback: object) -> None:
		"""Exit."""

	def run(self, obj: apt_pkg.PackageManager | bytes | str) -> int:
		"""Install using the `PackageManager` object `obj`.

		returns the result of calling `obj.do_install()`
		"""
		pid, self.child_fd = fork()
		if pid == 0:
			try:
				# We ignore this with mypy because the attr is there
				os._exit(obj.do_install()) # type: ignore[union-attr]
			except AttributeError:
				# nosec because this isn't really a security issue. We're just running dpkg
				# Also we need this line for installing local debs
				os._exit(os.spawnlp(os.P_WAIT, "dpkg", "dpkg", "-i", cast(str, obj))) # nosec
			# We need to catch every exception here.
			# If we don't the code continues in the child,
			# And bugs will be very confusing
			except Exception as err: # pylint: disable=broad-except
				sys.stderr.write(f"{err}\n")
				os._exit(1)

		self.child_pid = pid
		if arguments.raw_dpkg:
			return os.WEXITSTATUS(self.wait_child())
		# We use fdspawn from pexpect to interact with out dpkg pty
		# But we also subclass it to give it the interact method and setwindow
		self.child = AptExpect(self.child_fd, timeout=None)

		signal.signal(signal.SIGWINCH, self.sigwinch_passthrough)
		self.child.interact(self.pre_filter)
		return os.WEXITSTATUS(self.wait_child())

	def wait_child(self) -> int:
		"""Wait for child progress to exit."""
		(pid, res) = (0, 0)
		while True:
			try:
				(pid, res) = os.waitpid(self.child_pid, os.WNOHANG)
				if pid == self.child_pid:
					break
			except OSError as err:
				if err.errno == errno.ECHILD:
					break
				if err.errno != errno.EINTR:
					raise
			# Sleep for a short amount of time so we don't waste CPU waiting on the child
			sleep(0.01)
		return res

	def sigwinch_passthrough(self, _sig_dummy: int, _data_dummy: FrameType | None) -> None:
		"""Pass through sigwinch signals to dpkg."""
		buffer = struct.pack("HHHH", 0, 0, 0, 0)
		term_size = struct.unpack(
			'hhhh', fcntl.ioctl(term.STDIN, termios.TIOCGWINSZ , buffer)
		)
		if not self.child.closed:
			setwinsize(self.child_fd, term_size[0], term_size[1])

	def conf_check(self, rawline: bytes) -> None:
		"""Check if we get a conf prompt."""
		if b"Configuration file '" in rawline and b'is obsolete.' not in rawline:
			self.raw_init()

	def conf_end(self, rawline: bytes) -> bool:
		"""Check to see if the conf prompt is over."""
		return rawline == term.CRLF and (
			b'*** config.inc.php (Y/I/N/O/D/Z) [default=N] ?' in self.last_line
			or self.last_line in DPKG_MSG['CONF_ANSWER']
		)

	def dpkg_log(self, msg: str) -> None:
		"""Write to dpkg-debug.log and flush."""
		self._dpkg_log.write(msg)
		self._dpkg_log.flush()

	def dpkg_status(self, data: bytes) -> bool:
		"""Handle any status messages."""
		for status in DPKG_MSG['DPKG_STATUS']:
			if status in data:
				statuses = data.split(b'\r')
				if len(statuses) > 2:
					self.dpkg_log(f"Status_Split = {repr(statuses)}\n")
				for msg in statuses:
					if msg != b'':
						spinner.text = Text.from_ansi(
							color(msg.decode().strip())
						)
						scroll_bar(self, update_spinner=True)
				self.dpkg_log(term.LF.decode())
				return True
		return False

	def pre_filter(self, data: bytes) -> None:
		"""Filter data from interact."""
		# Set to raw if we have a conf prompt
		self.conf_check(data)

		# Save Term for debconf and Bracked Paste for the start of the shell
		if term.SAVE_TERM in data or term.ENABLE_BRACKETED_PASTE in data:
			self.raw_init()

		self.dpkg_log(f"Raw = {self.raw}: [{repr(data)}]\n")

		if not self.raw:
			if self.dpkg_status(data):
				return

			if not data.endswith(term.CRLF):
				self.line_fix.append(data)
				return

			if self.line_fix:
				self.dpkg_log(f"line_fix = {repr(self.line_fix)}\n")
				data = b''.join(self.line_fix) + data
				self.line_fix.clear()

			if data.count(b'\r\n') > 1:
				self.split_data(data)
				return
		self.format_dpkg_output(data)

	def split_data(self, data: bytes) -> None:
		"""Split data into clean single lines to format."""
		data_split = data.split(b'\r\n')
		self.dpkg_log(f"Data_Split = {repr(data_split)}\n")
		for line in data_split:
			if line != b'':
				self.format_dpkg_output(line)

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
		line = rawline.decode().strip()

		if check_line_spam(line, rawline):
			return

		if line == '':
			return
		# Main format section for making things pretty
		msg = msg_formatter(line)
		self.advance_progress(line)
		# If verbose we just send it. No bars
		if arguments.verbose:
			print(msg)
		else:
			scroll_bar(self, msg)

		self.set_last_line(rawline)

	def rawline_handler(self, rawline: bytes) -> None:
		"""Handle text operations for rawline."""
		term.write(rawline)
		# Once we write we can check if we need to pop out of raw mode
		if term.RESTORE_TERM in rawline or self.conf_end(rawline):
			self.raw = False
			term.restore_mode()
			self.live.start()
		self.set_last_line(rawline)

	def set_last_line(self, rawline: bytes) -> None:
		"""Set the current line to last line if there is no backspace."""
		# When at the conf prompt if you press Y, then backspace, then hit enter
		# Things get really buggy so instead we check for a backspace
		if term.BACKSPACE not in rawline:
			self.last_line = rawline

	def advance_progress(self, line: str) -> None:
		"""Advance the dpkg progress bar."""
		if ('Setting up' in line or 'Unpacking' in line
			or 'Removing' in line and '(' in line):
			dpkg_progress.advance(self.task)
		if arguments.verbose:
			self.live.update(
				Panel.fit(
					dpkg_progress.get_renderable(), border_style='bold green', padding=(0,0)
				), refresh=True
			)

	def raw_init(self) -> None:
		"""Initialize raw terminal output."""
		# We update the live display to blank. Then move up 1 and clear
		# This prevents weird artifacts from the progress bar after debconf prompts
		if self.raw:
			return
		self.live.update('', refresh=True)
		term.write(term.CURSER_UP+term.CLEAR_LINE)
		self.live.stop()
		term.set_raw()
		self.raw = True

def check_line_spam(line: str, rawline: bytes) -> bool:
	"""Check for, and handle, notices and spam."""
	for message in DPKG_MSG['NOTICES']:
		if message in rawline:
			notice.add(line)
			return False

	return any(item in line for item in SPAM)

def paren_color(match: Match[str]) -> str:
	"""Color parenthesis."""
	return color('(') if match.group(0) == '(' else color(')')

def lines(line: str, zword: str, msg_color: str) -> str:
	"""Color and space our line."""
	space = ' '
	if zword == 'Removing':
		space *= 3
	elif zword == 'Unpacking':
		space *= 2
	return line.replace(zword, color(f'{zword}:{space}', msg_color))

def format_version(match: list[str], line: str) -> str:
	"""Format version numbers."""
	for ver in match:
		version = ver[1:-1]
		if version[0].isdigit():
			new_ver = ver.replace(version, color(version, 'BLUE'))
			new_ver = re.sub(PARENTHESIS_PATTERN, paren_color, new_ver)
			line = line.replace(ver, new_ver)
	return line

def fill_pulse(pulse: list[str]) -> str:
	"""Fill the pulse message."""
	last = len(pulse) - 1
	fill = sum(len(line) for line in pulse) + last
	# Set fill width to fit inside the rich panel
	fill = ((term.columns - fill) - 5 if arguments.verbose else
			(term.columns - fill) - 7)
	# Make sure we insert the filler in the right spot
	# In case of extra 1 min as shown below.
	# ['2407', 'kB/s', '30s']
	# ['895', 'kB/s', '1min', '18s']
	index = last-2
	if '/s' in pulse[index]:
		index = last-3

	pulse.insert(index, ' '*fill)
	return ' '.join(pulse)

def msg_formatter(line: str) -> str:
	"""Format dpkg output."""
	if line.endswith('...'):
		line = line.replace('...', '')

	if line.startswith('Removing'):
		line = lines(line, 'Removing', 'RED')
	elif line.startswith('Unpacking'):
		line = lines(line, 'Unpacking', 'GREEN')
	elif line.startswith('Setting up'):
		line = lines(line, 'Setting up', 'GREEN')
	elif line.startswith('Processing'):
		line = lines(line, 'Processing', 'GREEN')

	if match := re.findall(VERSION_PATTERN, line):
		return format_version(match, line)
	return line

def get_title(install: bool, fetch: bool) -> str:
	"""Get the title for our panel."""
	if arguments.command and install and not fetch:
		if arguments.command in ('remove', 'purge'):
			return '[bold white]Removing Packages'
		if arguments.command in ('update', 'upgrade'):
			return '[bold white]Updating Packages'
		if arguments.command == 'install':
			return '[bold white]Installing Packages'
		if arguments.command == 'history':
			title = f'History {str(arguments.mode).capitalize()} {arguments.id}'
			return '[bold white]' + title
	if install and fetch:
		return '[bold white]Fetching Missed Packages'
	return '[bold white]Updating Package List'

def get_group(update_spinner: bool, use_bar: bool) -> Group:
	"""Get the group for our panel."""
	if update_spinner and use_bar:
		return Group(
			spinner,
			dpkg_progress.get_renderable(),
		)
	if update_spinner:
		return Group(
			spinner
		)
	return Group(
		dpkg_progress.get_renderable(),
	)

def slice_list() -> None:
	"""Set scroll bar to take up only 1/3 of the screen."""
	global scroll_list # pylint: disable=invalid-name, global-statement
	scroll_lines = term.lines // 3
	size = len(scroll_list)
	if size > scroll_lines and size > 10:
		total = size - max(scroll_lines, 10)
		scroll_list = scroll_list[total:]

def scroll_bar(self: UpdateProgress | InstallProgress, # pylint: disable=too-many-arguments
	msg: str = '', install: bool = True, fetch: bool = False,
	update_spinner: bool = False, use_bar: bool = True) -> None:
	"""Print msg to our scroll bar live display."""
	if msg:
		scroll_list.append(msg)

	slice_list()
	scroll_title = get_title(install, fetch)

	bar_style = 'bold green' if arguments.verbose else 'bold blue'
	table = Table.grid()
	table.add_column(no_wrap=True, width=term.columns)
	for item in scroll_list:
		table.add_row(Text.from_ansi(item))

	panel_group = get_group(update_spinner, use_bar)

	if use_bar or update_spinner:
		table.add_row(Panel(panel_group, padding=(0,0), border_style=bar_style))
	# We don't need to build the extra panel if we're not scrolling
	if arguments.verbose:
		self.live.update(table, refresh=True)
		return
	self.live.update(Panel(
		table, title=scroll_title, title_align='left',
		padding=(0,0), border_style='bold green'
		), refresh=True)

def setwinsize(file_descriptor: int, rows: int, cols: int) -> None:
	"""Set the terminal window size of the child tty.

	This will cause a SIGWINCH signal to be sent to the child. This does not
	change the physical window size. It changes the size reported to
	TTY-aware applications like vi or curses -- applications that respond to
	the SIGWINCH signal.
	"""
	tiocswinz = getattr(termios, 'TIOCSWINSZ', -2146929561)
	# Note, assume ws_xpixel and ws_ypixel are zero.
	size = struct.pack('HHHH', rows, cols, 0, 0)
	fcntl.ioctl(file_descriptor, tiocswinz, size)

def fork() -> tuple[int, int]:
	"""Fork pty or regular."""
	return (os.fork(), 0) if arguments.raw_dpkg else pty.fork()

class AptExpect(fdspawn): # type: ignore[misc]
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

		setwinsize(self.child_fd, term.lines, term.columns)

		try:
			self.interact_copy(output_filter)
		finally:
			term.restore_mode()

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
				term.write(term.CURSER_UP+term.CLEAR_LINE)
				print(
					color("Warning:", 'YELLOW'),
					"Quitting now could break your system!"
				)
				print(color("Ctrl+C twice quickly will exit...", 'RED'))
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
		if data == b'':
			# BSD-style EOF
			return False
		output_filter(data)
		return True

	def _write(self) -> None:
		"""Write user inputs into the pty."""
		data = os.read(term.STDIN, 1000)
		while data != b'' and self.isalive():
			split = os.write(self.child_fd, data)
			data = data[split:]
