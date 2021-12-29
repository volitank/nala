import os
import sys
import re
import fcntl
import errno
import signal
import tty
import apt
import apt_pkg
from pathlib import Path
from pty import STDIN_FILENO, STDOUT_FILENO, fork
from pexpect.fdpexpect import fdspawn
from pexpect.utils import poll_ignore_interrupts, errno
from ptyprocess.ptyprocess import _setwinsize
from shutil import get_terminal_size
from typing import Union
import apt.progress.base as base
import apt.progress.text as text
from click import style

from nala.rich_custom import rich_live, rich_grid, rich_spinner
from nala.utils import (
	# Import Style Colors
	RED, BLUE, GREEN, YELLOW,
	# Import Message
	CONF_MESSAGE, CONF_ANSWER, NOTICES, SPAM,
	# Lonely Import File :(
	DPKG_LOG,
)

# Control Codes
CURSER_UP = b'\x1b[1A'
CLEAR_LINE = b'\x1b[2k'

# Overriding apt cache so we can make it exit on ctrl+c
class nalaCache(apt.Cache):
	def update(self, fetch_progress=None, pulse_interval=0,
			   raise_on_error=True, sources_list=None):
		"""Run the equivalent of apt-get update.

		You probably want to call open() afterwards, in order to utilise the
		new cache. Otherwise, the old cache will be used which can lead to
		strange bugs.

		The first parameter *fetch_progress* may be set to an instance of
		apt.progress.FetchProgress, the default is apt.progress.FetchProgress()
		.
		sources_list -- Update a alternative sources.list than the default.
		Note that the sources.list.d directory is ignored in this case
		"""
		from apt.cache import _WrappedLock, FetchFailedException
		with _WrappedLock(apt_pkg.config.find_dir("Dir::State::Lists")):
			if sources_list:
				old_sources_list = apt_pkg.config.find("Dir::Etc::sourcelist")
				old_sources_list_d = (
					apt_pkg.config.find("Dir::Etc::sourceparts"))
				old_cleanup = apt_pkg.config.find("APT::List-Cleanup")
				apt_pkg.config.set("Dir::Etc::sourcelist",
									Path(sources_list).absolute()
				)
				apt_pkg.config.set("Dir::Etc::sourceparts", "xxx")
				apt_pkg.config.set("APT::List-Cleanup", "0")
				slist = apt_pkg.SourceList()
				slist.read_main_list()
			else:
				slist = self._list

			try:
				if fetch_progress is None:
					fetch_progress = base.AcquireProgress()
				try:
					res = self._cache.update(fetch_progress, slist,
											 pulse_interval)
				except SystemError as e:
					print(
						style('Error:', **RED),
						str(e).replace('E:', ''),
						'\nCannot continue..'
					)
					exit(1)
				# If told to exit please do so
				except KeyboardInterrupt as e:
					exit(1)
				if not res and raise_on_error:
					raise FetchFailedException()
				else:
					return res
			finally:
				if sources_list:
					apt_pkg.config.set("Dir::Etc::sourcelist",
									   old_sources_list)
					apt_pkg.config.set("Dir::Etc::sourceparts",
									   old_sources_list_d)
					apt_pkg.config.set("APT::List-Cleanup",
									   old_cleanup)

# Override the text progress to format updating output
# This is mostly for `apt update`
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

	def _(msg):
		# Not really sure what the point of this is. If anyone can help out
		# I would appreciate it. Leaving it here anyway and will comment the
		# original lines in the methods using it
		# type: (str) -> str
		"""Translate the message, also try apt if translation is missing."""
		res = apt_pkg.gettext(msg)
		if res == msg:
			res = apt_pkg.gettext(msg, "apt")
		return res

	def _write(self, msg, newline=True, maximize=False):
		# type: (str, bool, bool) -> None
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
		try:
			base.AcquireProgress.stop(self)
		except KeyboardInterrupt:
			exit()
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
	def __init__(self, verbose: bool = False, debug: bool = False):

		self.verbose = verbose
		self.debug = debug
		self.raw = False
		self.raw_dpkg = False
		self.last_line = None
		self.xterm = os.environ["TERM"]
		self.live = rich_live(redirect_stdout=False)
		self.spinner = rich_spinner('dots', text='Initializing dpkg', style="bold blue")
		self.scroll = [self.spinner]

		if 'xterm' in self.xterm:
			self.xterm = True
			self.debconf_start = b'\x1b[22;0;0t'
			self.debconf_stop = b'\x1b[23;0;0t'
		else:
			self.xterm = False
			# Hide cursor sequence. Determines start of debconf
			self.debconf_start = b'\x1b[?25l'
			# # Restore cursor, not really reliable for detecting end
			# b'\x1b[?25h'
			#debconf_stop = b'\x1b[m\x0f\x1b[37m\x1b[40m\x1b[m\x0f\x1b[39;49m\r\x1b[K\r'
			# We may end up having to detect more than one thing for the end.
			# Checking if not \x1b doesn't really work because conf files and aptlist pager prints
			# Some lines that don't have escape sequences. Doing so breaks those 
			self.debconf_stop = b'\x1b[40m\x1b[m\x0f\x1b[39;49m\r\x1b[K\r'
		self.apt_list_start = b'apt-listchanges:'
		self.apt_list_end = b'\r\x1b[K'

		(self.statusfd, self.writefd) = os.pipe()
		# These will leak fds, but fixing this safely requires API changes.
		self.write_stream = os.fdopen(self.writefd, "w")
		self.status_stream = os.fdopen(self.statusfd, "r")
		fcntl.fcntl(self.statusfd, fcntl.F_SETFL, os.O_NONBLOCK)

	def start_update(self):
		"""(Abstract) Start update."""
		self.notice = set()
		self.live.start()
		self.spinner.text = style('Running dpkg...', bold=True)

	def finish_update(self):
		"""(Abstract) Called when update has finished."""
		self.live.stop()
		if self.notice:
			print('\n'+style('Notices:', bold=True))
			for notice in self.notice:
				print(notice)
		print(style("Finished Successfully", **GREEN))

	def __exit__(self, type, value, traceback):
		self.write_stream.close()
		self.status_stream.close()
		self.dpkg.close()

	def run(self, dpkg):
		"""
		Install using the `PackageManager` object `dpkg`

		returns the result of calling `dpkg.do_install()`
		"""
		pid, self.fd = fork()

		if pid == 0:
			try:
				###########################
				## If we pass a fd here for status stream we MUST read from it
				## Using this and not reading from it causes a deadlock on dpkg output
				os._exit(dpkg.do_install())
				#os._exit(dpkg.do_install(self.write_stream.fileno()))
			except Exception as e:
				sys.stderr.write("%s\n" % e)
				os._exit(apt_pkg.PackageManager.RESULT_FAILED)

		self.child_pid = pid
		self.dpkg = os.fdopen(self.fd, "r")
		fcntl.fcntl(self.fd, fcntl.F_SETFL, os.O_NONBLOCK)

		# We use fdspawn from pexpect to interact with out dpkg pty
		# But we also subclass it to give it the interact method and setwindow
		self.child = dcexpect(self.dpkg, timeout=None)

		signal.signal(signal.SIGWINCH, self.sigwinch_passthrough)
		with open(DPKG_LOG, 'w') as self.dpkg_log:
			#with open(DPKG_STATUS_LOG, 'w') as self.dpkg_status:
			self.child.interact(output_filter=self.format_dpkg_output)

		# This is really just here so dpkg exits properly
		res = self.wait_child()
		return os.WEXITSTATUS(res)

	def sigwinch_passthrough(self, sig, data):
		import struct, termios
		s = struct.pack("HHHH", 0, 0, 0, 0)
		a = struct.unpack('hhhh', fcntl.ioctl(sys.stdout.fileno(),
			termios.TIOCGWINSZ , s))
		if self.child.isalive():
			self.child.setwinsize(a[0],a[1])

	def conf_end(self, rawline):
		return rawline == b'\r\n' and (CONF_MESSAGE[9] in self.last_line
										or self.last_line in CONF_ANSWER)

	def format_dpkg_output(self, rawline: bytes):
		## Commenting out this for now. I basically only use this sometimes during development
		## It doesn't make sense to keep it in if it's not being used
		# try:
		# 	status = self.status_stream.readline()
		# except IOError as err:
		# 	# resource temporarly unavailable is ignored
		# 	if err.errno != errno.EAGAIN and err.errno != errno.EWOULDBLOCK:
		# 		print(err.strerror)
		# 	return
		# if status != '':
		# 	self.dpkg_status.write(status)
		# 	self.dpkg_status.flush()

		# During early development this is mandatory
		# if self.debug:
		# self.dpkg_log.write(repr(rawline)+'\n')
		# self.dpkg_log.flush()

		# These are real spammy the way we set this up
		# So if we're in verbose just send it
		if self.verbose:
			for line in (
				b'Reading changelogs...',
				b'Scanning processes...',
				b'Scanning candidates...',
				b'Scanning linux images...',):
				if line in rawline:
					os.write(STDOUT_FILENO, rawline)
					return

		# There isn't really an option to hit this yet
		# But eventually I will make --raw-dpkg switch
		if self.raw_dpkg:
			os.write(STDOUT_FILENO, rawline)
			return

		if not self.xterm and self.apt_list_start in rawline:
			self.raw = True

		# I wish they would just use debconf for this.
		# But here we are and this is what we're doing for config files
		for line in CONF_MESSAGE:
			# We only iterate the whole list just in case. We don't want to miss this.
			# Even if we just hit the last line it's better than not hitting it.
			if line in rawline:
				# Sometimes dpkg be like yo I'm going to say the same thing as the conf prompt
				# But a little different so it will trip you up.
				if rawline.endswith((b'.', b'\r\n')):
					break
				self.raw = True
				# Add return because our progress bar might eat one
				#if not rawline.startswith(b'\r'):
				rawline = b'\r'+rawline
				break

		# This second one is for the start of the shell
		if self.debconf_start in rawline or b'\x1b[?2004h' in rawline:
			self.raw = True

		if self.raw:
			##################################
			## Starting Raw we probably want to remove the live display
			## And clear everything to prepare for raw text. IDK how it handles this
			## Will need testing with and without.
			self.live.update('')
			control_code(CURSER_UP+CLEAR_LINE)
			self.live.stop()
			##
			os.write(STDOUT_FILENO, rawline)
			if (self.debconf_stop in rawline
				or self.conf_end(rawline)
				or (self.apt_list_start in rawline and self.apt_list_end == self.last_line)):
				self.raw = False
				self.live.start()
		else:
			line = rawline.decode().strip()
			if line == '':
				return
			for message in NOTICES:
				if message in rawline:
					self.notice.add(line)
					break
			for item in SPAM:
				if item in line:
					break
			else:
				# Main format section for making things pretty
				msg = msg_formatter(line)
				# If verbose we just send it. No bars
				if self.verbose:
					# We have to append Carrige return and new line or things get weird
					os.write(STDOUT_FILENO, (msg+'\r\n').encode())
				else:
					# Handles our scroll_bar effect
					#os.write(STDOUT_FILENO, (msg+'\r\n').encode())
					scroll_bar(self, msg)
		# Just something because if you do Y, then backspace, then hit enter
		# At the conf prompt it'll get buggy
		if b'\x08' not in rawline:
			self.last_line = rawline
		
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
	self.scroll.append(msg)

	self.scroll.append(
		self.scroll.pop(
			self.scroll.index(self.spinner)
		)
	)

	if len(self.scroll) > 10:
		del self.scroll[0]

	table = rich_grid()
	for item in self.scroll:
		table.add_row(item)

	self.live.update(table)

def control_code(code):
	"""Wrapper for sending escape codes"""
	os.write(STDIN_FILENO, code)

class dcexpect(fdspawn):
	def interact(self, output_filter=None):
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
		mode = tty.tcgetattr(STDIN_FILENO)
		

		cols, rows = get_terminal_size()
		self.setwinsize(rows, cols)

		try:
			self.__interact_copy(output_filter, mode)
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

	def __interact_copy(self, output_filter, mode):
		'''This is used by the interact() method.
		'''
		while self.isalive():
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
				tty.setraw(STDIN_FILENO)
				data = os.read(STDIN_FILENO, 1000)
				while data != b'' and self.isalive():
					n = os.write(self.child_fd, data)
					data = data[n:]
				tty.tcsetattr(STDIN_FILENO, tty.TCSAFLUSH, mode)
