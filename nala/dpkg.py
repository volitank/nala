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

import errno
import fcntl
import os
import re
import signal
import sys
import tty
from pty import STDIN_FILENO, STDOUT_FILENO, fork
from shutil import get_terminal_size
from time import sleep
from typing import Union

import apt.progress.base as base
import apt.progress.text as text
import apt_pkg
from click import style
from pexpect.fdpexpect import fdspawn
from pexpect.utils import errno, poll_ignore_interrupts
from ptyprocess.ptyprocess import _setwinsize

from nala.rich_custom import rich_grid, rich_live, rich_spinner
from nala.utils import (  # Import Style Colors; Import Message; Lonely Import File :(
	BLUE, CONF_ANSWER, CONF_MESSAGE, DPKG_LOG,
	DPKG_STATUS, GREEN, NOTICES, RED, SPAM, YELLOW,)

# Control Codes
CURSER_UP = b'\x1b[1A'
CLEAR_LINE = b'\x1b[2k'
CLEAR_FROM_CURRENT_TO_END = b'\x1b[K'
BACKSPACE = b'\x08'
ENABLE_BRACKETED_PASTE = b'\x1b[?2004h'
DISABLE_BRACKETED_PASTE = b'\x1b[?2004l'
ENABLE_ALT_SCREEN = b'\x1b[?1049h'
DISABLE_ALT_SCREEN = b'\x1b[?1049l'
SHOW_CURSOR = b'\x1b[?25h'
HIDE_CURSOR = b'\x1b[?25l'
SAVE_TERM = b'\x1b[22;0;0t'
RESTORE_TERM = b'\x1b[23;0;0t'
APPLICATION_KEYPAD = b'\x1b='
NORMAL_KEYPAD = b'\x1b>'
CR = b'\r'
LF = b'\n'

class nalaProgress(text.AcquireProgress, base.OpProgress):

	def __init__(self, verbose=False, debug=False):
		text.TextProgress.__init__(self)
		base.AcquireProgress.__init__(self)

		self._file = sys.stdout
		self.live = rich_live(redirect_stdout=False)
		self.spinner = rich_spinner('dots', text='Initializing Cache', style="bold blue")
		self.scroll = [self.spinner]
		self._signal = None
		self._width = 80
		self._id = 1
		self.verbose=verbose
		self.debug=debug
		self.termsize = get_terminal_size()

		# OpProgress settings
		base.OpProgress.__init__(self)
		self.old_op = ""

	# OpProgress Method
	def update(self, percent=None):
		"""Called periodically to update the user interface."""
		base.OpProgress.update(self, percent)
		if self.verbose:
			if self.major_change and self.old_op:
				self._write(self.old_op)
			self._write("%s... %i%%\r" % (self.op, self.percent), False, True)
			self.old_op = self.op

	# OpProgress Method
	def done(self, compatibility=None):
		"""Called once an operation has been completed."""
		base.OpProgress.done(self)
		if self.verbose:
			if self.old_op:
				self._write(text._("%c%s... Done") % ('\r', self.old_op), True, True)
			self.old_op = ""

	def _write(self, msg, newline=True, maximize=False):
		"""Write the message on the terminal, fill remaining space."""

		if self.verbose:
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
		else:
			for item in ['Updated:', 'Ignored:', 'Error:', 'No Change:']:
				if item in msg:
					scroll_bar(self, msg)
					break
			else:
				# For the pulse messages we need to do some formatting
				# End of the line will look like '51.8 mB/s 2s'
				if msg.endswith('s'):
					msg = msg.split()
					last = len(msg) - 1
					fill = sum(len(line) for line in msg) + last
					# Minus three too account for our spinner dots
					fill = (self._width - fill) - 3
					msg.insert(last-2, ' '*fill)
					msg = ' '.join(msg)

				self.spinner.text = msg

	def ims_hit(self, item):
		"""Called when an item is update (e.g. not modified on the server)."""
		base.AcquireProgress.ims_hit(self, item)
		no_change = style('No Change:', **GREEN)
		line = f"{no_change} {item.description}"
		if item.owner.filesize:
			size = apt_pkg.size_to_str(item.owner.filesize)
			line += f' [{size}B]'
		self._write(line)

	def fail(self, item):
		"""Called when an item is failed."""
		base.AcquireProgress.fail(self, item)
		if item.owner.status == item.owner.STAT_DONE:
			ignored = style('Ignored:  ', **YELLOW)
			self._write(f"{ignored} {item.description}")
		else:
			err = style('Error:    ', **RED)
			self._write(f"{err} {item.description}")
			self._write(f"  {item.owner.error_text}")

	def fetch(self, item):
		"""Called when some of the item's data is fetched."""
		base.AcquireProgress.fetch(self, item)
		# It's complete already (e.g. Hit)
		if item.owner.complete:
			return
		update = style('Updated:  ', **BLUE)
		line = f"{update} {item.description}"
		if item.owner.filesize:
			size = apt_pkg.size_to_str(item.owner.filesize)
			line += f" [{size}B]"
		self._write(line)

	def start(self):
		"""Start an Acquire progress.

		In this case, the function sets up a signal handler for SIGWINCH, i.e.
		window resize signals. And it also sets id to 1.
		"""
		base.AcquireProgress.start(self)
		self._signal = signal.signal(signal.SIGWINCH, self._winch)
		# Get the window size.
		self._winch()
		self._id = 1
		self.live.start()

	def stop(self):
		"""Invoked when the Acquire process stops running."""
		base.AcquireProgress.stop(self)
		# Trick for getting a translation from apt
		fetched = apt_pkg.size_to_str(self.fetched_bytes)
		elapsed = apt_pkg.time_to_str(self.elapsed_time)
		speed = apt_pkg.size_to_str(self.current_cps).rstrip("\n")
		msg = style(f"Fetched {fetched}B in {elapsed} ({speed}B/s)", bold=True)
		self._write(msg)

		# Delete the signal again.
		signal.signal(signal.SIGWINCH, self._signal)
		self.live.stop()

class InstallProgress(base.InstallProgress):
	def __init__(self,
		verbose: bool = False,
		debug: bool = False,
		raw_dpkg: bool = False):

		self.verbose = verbose
		self.debug = debug
		self.raw_dpkg = raw_dpkg
		self.raw = False

		self.last_line = None
		self.termsize = get_terminal_size()
		self.live = rich_live(redirect_stdout=False)
		self.spinner = rich_spinner('dots', text='Initializing dpkg', style="bold blue")
		self.mode = tty.tcgetattr(STDIN_FILENO)
		self.scroll = [self.spinner]

		if self.raw_dpkg:
			tty.setraw(STDIN_FILENO)

		# setting environment to xterm seems to work find for linux terminal
		# I don't think we will be supporting much more this this, at least for now
		if 'xterm' not in os.environ["TERM"]:
			os.environ["TERM"] = 'xterm'

	def start_update(self):
		"""Start update."""
		self.notice = set()
		if not self.verbose and not self.raw_dpkg:
			self.live.start()
			self.spinner.text = style('Initializing dpkg...', **BLUE)

	def finish_update(self):
		"""Called when update has finished."""
		if not self.verbose and not self.raw_dpkg:
			self.live.stop()
		if self.notice:
			print('\n'+style('Notices:', bold=True))
			for notice in self.notice:
				print(notice)
		print(style("Finished Successfully", **GREEN))

	def __exit__(self, type, value, traceback):
		pass

	def run(self, dpkg):
		"""
		Install using the `PackageManager` object `dpkg`

		returns the result of calling `dpkg.do_install()`
		"""
		pid, self.fd = fork()

		if pid == 0:
			try:
				os._exit(dpkg.do_install())
			except Exception as e:
				sys.stderr.write("%s\n" % e)
				os._exit(apt_pkg.PackageManager.RESULT_FAILED)

		self.child_pid = pid
		fcntl.fcntl(self.fd, fcntl.F_SETFL, os.O_NONBLOCK)

		# We use fdspawn from pexpect to interact with out dpkg pty
		# But we also subclass it to give it the interact method and setwindow
		self.child = dcexpect(self.fd, timeout=None)

		signal.signal(signal.SIGWINCH, self.sigwinch_passthrough)
		with open(DPKG_LOG, 'w') as self.dpkg_log:
			#with open(DPKG_STATUS_LOG, 'w') as self.dpkg_status:
			try:
				self.child.interact(
					self.format_dpkg_output,
					self
					)
			finally:
				# We need to make sure that no matter what the terminal
				# Settings are restored if for some reason we stop
				tty.tcsetattr(STDIN_FILENO, tty.TCSAFLUSH, self.mode)

		return os.WEXITSTATUS(0)

	def sigwinch_passthrough(self, sig, data):
		import struct
		import termios
		s = struct.pack("HHHH", 0, 0, 0, 0)
		a = struct.unpack('hhhh', fcntl.ioctl(STDOUT_FILENO,
			termios.TIOCGWINSZ , s))
		if not self.child.closed:
			self.child.setwinsize(a[0],a[1])

	def conf_check(self, rawline):
		"""Checks if we get a conf prompt"""
		# I wish they would just use debconf for this.
		# But here we are and this is what we're doing for config files
		for line in CONF_MESSAGE:
			# We only iterate the whole list just in case. We don't want to miss this.
			# Even if we just hit the last line it's better than not hitting it.
			if line in rawline:
				# Sometimes dpkg be like yo I'm going to say the same thing as the conf prompt
				# But a little different so it will trip you up.
				if rawline.endswith((b'.', CR+LF)):
					break
				self.raw = True
				self.raw_init()
				# Add return because our progress bar might eat one
				#if not rawline.startswith(b'\r'):
				rawline = CR+rawline
				break

	def conf_end(self, rawline):
		return rawline == CR+LF and (CONF_MESSAGE[9] in self.last_line
										or self.last_line in CONF_ANSWER)

	def format_dpkg_output(self, rawline: bytes):
		# During early development this is mandatory
		# if self.debug:
		self.dpkg_log.write(repr(rawline)+'\n')
		self.dpkg_log.flush()

		if self.raw_dpkg:
			os.write(STDOUT_FILENO, rawline)
			return

		# These are real spammy the way we set this up
		# So if we're in verbose just send it
		for item in DPKG_STATUS:
			if item in rawline:
				if self.verbose:
					os.write(STDOUT_FILENO, rawline)
				else:
					self.spinner.text = style(
						rawline.decode().strip(), bold=True
					)
					scroll_bar(self, msg=None)
				return

		# Set to raw if we have a conf prompt
		self.conf_check(rawline)

		# This second one is for the start of the shell
		if SAVE_TERM in rawline or ENABLE_BRACKETED_PASTE in rawline:
			self.raw = True
			self.raw_init()

		if self.raw:
			self.rawline_handler(rawline)
			return

		self.line_handler(rawline)

	def line_handler(self, rawline: bytes):
		"""Handles text operations if we're not using a rawline"""
		line = rawline.decode().strip()
		if line == '':
			return

		if self.check_line_spam(line, rawline):
			return

		self.spinner.text = style('Running dpkg...', bold=True)
		# Main format section for making things pretty
		msg = msg_formatter(line)
		# If verbose we just send it. No bars
		if self.verbose:
			# We have to append Carriage return and new line or things get weird
			os.write(STDOUT_FILENO, (msg+'\r\n').encode())
		else:
			scroll_bar(self, msg)

		self.set_last_line(rawline)

	def rawline_handler(self, rawline):
		os.write(STDOUT_FILENO, rawline)
		# Once we write we can check if we need to pop out of raw mode
		if RESTORE_TERM in rawline or self.conf_end(rawline):
			self.raw = False
			tty.tcsetattr(STDIN_FILENO, tty.TCSAFLUSH, self.mode)
			self.live.start()
		self.set_last_line(rawline)
		return

	def set_last_line(self, rawline):
		# Just something because if you do Y, then backspace, then hit enter
		# At the conf prompt it'll get buggy if we don't check for backspace
		if BACKSPACE not in rawline:
			self.last_line = rawline

	def raw_init(self):
		self.live.update('')
		control_code(CURSER_UP+CLEAR_LINE)
		self.live.stop()
		tty.setraw(STDIN_FILENO)

	def check_line_spam(self, line, rawline):
		for message in NOTICES:
			if message in rawline:
				self.notice.add(line)
				break

		for item in SPAM:
			if item in line:
				return True

def msg_formatter(line):
	msg = ''
	line = line.split()
	for word in line:
		match = re.fullmatch(r'\(.*.\)', word)
		if word == 'Removing':
			msg += style('Removing:   ', **RED)
		elif word == 'Unpacking':
			msg += style('Unpacking:  ', **GREEN)
		elif word == 'Setting':
			msg += style('Setting ', **GREEN)
		elif word == 'up':
			msg += style('up: ', **GREEN)
		elif word == 'Processing':
			msg += style('Processing: ', **GREEN)
		elif word == '...':
			continue
		elif match:
			word = re.sub('[()]', '', word)
			paren = style('(', bold=True)
			paren2 = style(')', bold=True)
			msg += (' ') + paren+style(word, **BLUE)+paren2
		else:
			msg += ' ' + word
	return msg

def scroll_bar(self: Union[nalaProgress, InstallProgress], msg:str):
	"""self is either NalaProgress or InstallProgress. Msg is the Message"""
	if msg:
		self.scroll.append(msg)

	self.scroll.append(
		self.scroll.pop(
			self.scroll.index(self.spinner)
		)
	)

	# Set the scroll bar to take up a 3rd of the screen
	scroll_lines = self.termsize.lines // 3

	#if len(self.scroll) > 10:
	if len(self.scroll) > scroll_lines and len(self.scroll) > 10:
		del self.scroll[0]

	table = rich_grid()
	table.add_column(no_wrap=True)
	for item in self.scroll:
		table.add_row(item)

	self.live.update(table)

def control_code(code):
	"""Wrapper for sending escape codes"""
	os.write(STDIN_FILENO, code)

class dcexpect(fdspawn):
	def interact(self, output_filter, parent):
		"""Hacked up interact method because pexpect doesn't want to have one
		for fdspawn

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
		self.parent = parent
		mode = tty.tcgetattr(STDIN_FILENO)

		cols, rows = get_terminal_size()
		self.setwinsize(rows, cols)

		try:
			self.__interact_copy(output_filter)
		finally:
			tty.tcsetattr(STDIN_FILENO, tty.TCSAFLUSH, mode)

	def setwinsize(self, rows, cols):
		"""Set the terminal window size of the child tty.

		This will cause a SIGWINCH signal to be sent to the child. This does not
		change the physical window size. It changes the size reported to
		TTY-aware applications like vi or curses -- applications that respond to
		the SIGWINCH signal.
		"""
		return _setwinsize(self.child_fd, rows, cols)

	def __interact_copy(self, output_filter):
		'''This is used by the interact() method.
		'''
		while self.isalive():
			try:
				r = poll_ignore_interrupts([self.child_fd, STDIN_FILENO])
				if self.child_fd in r:
					try:
						data = os.read(self.child_fd, 1000)
					except OSError as err:
						if err.args[0] == errno.EIO:
							# Linux-style EOF
							break
						raise
					if data == b'':
						# BSD-style EOF
						break
					output_filter(data)
				if STDIN_FILENO in r:
					data = os.read(STDIN_FILENO, 1000)
					while data != b'' and self.isalive():
						n = os.write(self.child_fd, data)
						data = data[n:]
			except KeyboardInterrupt:
				err = style("Warning: ", **YELLOW)
				err += "quitting now could break your system!"
				if self.parent.live.is_started:
					self.parent.scroll.append(err)
					self.parent.scroll.append(style("Ctrl+C twice quickly will exit...", **RED))
					scroll_bar(self.parent, None)
				else:
					os.write(STDOUT_FILENO, LF+err.encode())
				sleep(0.5)
