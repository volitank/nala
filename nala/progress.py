import os
import sys
import re
import fcntl
import errno
import signal
import tty
import apt
import apt_pkg
from tqdm import tqdm
from pathlib import Path
from pty import STDIN_FILENO, STDOUT_FILENO
from pexpect.fdpexpect import fdspawn
from pexpect.utils import poll_ignore_interrupts, errno
from ptyprocess.ptyprocess import _setwinsize
from shutil import get_terminal_size
import apt.progress.base as base
import apt.progress.text as text
from click import style

from nala.utils import (
	# Import Style Colors
	RED, BLUE, GREEN, YELLOW,
	# Import Message
	CONF_MESSAGE, CONF_ANSWER, NOTICES, SPAM,
	# Import Files
	DPKG_LOG, DPKG_STATUS_LOG,
	dprint, ask
)

# Overriding apt cache so we can make it exit on ctrl+c
class nalaCache(apt.Cache):
	def update(self, fetch_progress=None, pulse_interval=0,
			   raise_on_error=True, sources_list=None):
		## type: (Optional[AcquireProgress], int, bool, Optional[str]) -> int
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

		self._signal = None
		self._width = 80
		self._id = 1
		self.counter = 1
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
		# Compatibility is just defined so we don't error while combining the two
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
				self.progress.set_description_str(f"{style(msg.strip(), bold=True)}")

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
		# type: (apt_pkg.AcquireItemDesc) -> None
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
		# type: () -> None
		"""Start an Acquire progress.

		In this case, the function sets up a signal handler for SIGWINCH, i.e.
		window resize signals. And it also sets id to 1.
		"""
		base.AcquireProgress.start(self)
		self._signal = signal.signal(signal.SIGWINCH, self._winch)
		# Get the window size.
		self._winch()
		self._id = 1
		# This is where we create our progress bars
		if not self.verbose:
			self.progress = tqdm(position=3, bar_format='{desc}', dynamic_ncols=True)
			self.update_log1 = tqdm(position=2, bar_format='{desc}', dynamic_ncols=True)
			self.update_log2 = tqdm(position=1, bar_format='{desc}', dynamic_ncols=True)
			self.update_log3 = tqdm(position=0, bar_format='{desc}', dynamic_ncols=True)

	def stop(self):
		# type: () -> None
		"""Invoked when the Acquire process stops running."""
		try:
			base.AcquireProgress.stop(self)
		except KeyboardInterrupt:
			exit()
		# Trick for getting a translation from apt
		fetched = apt_pkg.size_to_str(self.fetched_bytes)
		elapsed = apt_pkg.time_to_str(self.elapsed_time)
		speed = apt_pkg.size_to_str(self.current_cps).rstrip("\n")
		msg = f"Fetched {fetched}B in {elapsed} ({speed}B/s)"
		msg = style(msg, bold=True)
		if self.verbose:
			self._write(msg)
		else:
			self.progress.set_description_str(f"{msg}")

		# Delete the signal again.
		signal.signal(signal.SIGWINCH, self._signal)
		if not self.verbose:
			self.update_log3.close()
			self.update_log2.close()
			self.update_log1.close()
			self.progress.close()

class InstallProgress(base.InstallProgress):
	def __init__(self, verbose: bool = False,
						debug: bool = False):
		self.verbose = verbose
		self.debug = debug
		self.counter = 1
		self.raw = False
		self.raw_dpkg = False
		self.last_line = None
		self.xterm = os.environ["TERM"]

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

		if self.raw_dpkg:
			# This is just an easy way to disable progress bars
			# Verbose doesn't really do anything else if raw_dpkg is enabled
			self.verbose = True
		
		(self.statusfd, self.writefd) = os.pipe()
		# These will leak fds, but fixing this safely requires API changes.
		self.write_stream = os.fdopen(self.writefd, "w")
		self.status_stream = os.fdopen(self.statusfd, "r")
		fcntl.fcntl(self.statusfd, fcntl.F_SETFL, os.O_NONBLOCK)

	def start_update(self):
		"""(Abstract) Start update."""
		self.notice = set()
		self.progress_bars() # Create pkg bars

	def progress_bars(self, remove: bool=False, wipe=False):
		"Creates the progress bars. If remove=True it removes them instead."
		if self.verbose or self.debug:
			return
		if not remove:
			self.update_log1 = tqdm(position=2, bar_format='{desc}', dynamic_ncols=True)
			self.update_log2 = tqdm(position=1, bar_format='{desc}', dynamic_ncols=True)
			self.update_log3 = tqdm(position=0, bar_format='{desc}', dynamic_ncols=True)
		else:
			if wipe:
				self.update_log3.leave = False
				self.update_log2.leave = False
				self.update_log1.leave = False
			self.update_log3.close()
			self.update_log2.close()
			self.update_log1.close()

	def finish_update(self):
		# type: () -> None
		"""(Abstract) Called when update has finished."""
		self.progress_bars(remove=True)
		if self.notice:
			print('\n'+style('Notices:', bold=True))
			for notice in self.notice:
				print(notice)
		print(style("Finished Successfully", **GREEN))

	def __exit__(self, type, value, traceback):
		# type: (object, object, object) -> None
		self.write_stream.close()
		self.status_stream.close()
		self.dpkg.close()

	def run(self, obj):
		"""Install using the object 'obj'.

		This functions runs install actions. The parameter 'obj' may either
		be a PackageManager object in which case its do_install() method is
		called or the path to a deb file.

		If the object is a PackageManager, the functions returns the result
		of calling its do_install() method. Otherwise, the function returns
		the exit status of dpkg. In both cases, 0 means that there were no
		problems.
		"""
		pid, self.fd = os.forkpty()

		if pid == 0:
			try:
				# PEP-446 implemented in Python 3.4 made all descriptors
				# CLOEXEC, but we need to be able to pass writefd to dpkg
				# when we spawn it
				os.set_inheritable(self.writefd, True)
			except AttributeError:  # if we don't have os.set_inheritable()
				pass
			# pm.do_install might raise a exception,
			# when this happens, we need to catch
			# it, otherwise os._exit() is not run
			# and the execution continues in the
			# parent code leading to very confusing bugs
			try:
				os._exit(obj.do_install(self.write_stream.fileno()))  # type: ignore # noqa
			except AttributeError:
				os._exit(os.spawnlp(os.P_WAIT, "dpkg", "dpkg", "--status-fd",
								str(self.write_stream.fileno()), "-i",
								obj))  # type: ignore # noqa
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
		if rawline == b'\r\n':
			if CONF_MESSAGE[9] in self.last_line:
				return True
			elif self.last_line in CONF_ANSWER:
				return True
		return False

	def format_dpkg_output(self, rawline: bytes):
		msg = ''

		try:
			status = self.status_stream.readline()
		except IOError as err:
			# resource temporarly unavailable is ignored
			if err.errno != errno.EAGAIN and err.errno != errno.EWOULDBLOCK:
				print(err.strerror)
			return

		if True:
		# Change this to self.debug
		# During early development this is mandatory
			self.dpkg_log.write(repr(rawline)+'\n')
			self.dpkg_log.flush()
			# There isn't a ton of situations where we need this
			# if status != '':
			# 	self.dpkg_status.write(status)
			# 	self.dpkg_status.flush()

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

		else:
			if not self.xterm:
				# We have to handle the pager differently if we're not xterm
				# It just works^tm with xterm. They use b'\x1b[22;0;0t'
				if self.apt_list_start in rawline:
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
				self.progress_bars(remove=True, wipe=True)
				os.write(STDOUT_FILENO, rawline)
				if (self.debconf_stop in rawline
					or self.conf_end(rawline)
					or (self.apt_list_start in rawline and self.apt_list_end == self.last_line)):

					self.raw = False
					self.progress_bars()

			else:
				line = rawline.decode().strip()
				if line != '':

					for message in NOTICES:
						if message in rawline:
							self.notice.add(rawline.decode().strip())
							break

					for item in SPAM:
						if item in line:
							break
					else:
						# Main format section for making things pretty
						line = line.split()
						for word in line:
							match = re.fullmatch('\(.*.\)', word)
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
						# If verbose we just send it. No bars
						if self.verbose:
							# We have to append Carrige return and new line or things get weird
							os.write(STDOUT_FILENO, (msg+'\r\n').encode())
						else:
							# Handles our scroll_bar effect
							scroll_bar(self, msg.strip())
		# Just something because if you do Y, then backspace, then hit enter
		# At the conf prompt it'll get buggy
		if b'\x08' not in rawline:
			self.last_line = rawline

def scroll_bar(self, msg):
	"""self is either NalaProgress or InstallProgress. Msg is the Message"""
	if self.counter == 4:
		self.counter = 3
	# If our message is longer than our progress bar we need to truncate it
	# Otherwise the terminal gets spammed and lines wrap weird
	# We care more about being beautiful than verbosity in this mode
	if len(msg) > self.update_log1.ncols:
		msg = msg[:self.update_log1.ncols].strip()
	else:
		buff_num = self.update_log1.ncols - len(msg)
		buffer = ' '*(buff_num - 1)
		msg += buffer

	if self.counter == 1:
		self.old = msg
		self.update_log1.set_description_str(self.old)
		self.counter += 1

	elif self.counter == 2:
		self.update_log1.set_description_str(msg)
		self.update_log2.set_description_str(self.old)

		self.old_old = self.old
		self.old = msg
		self.counter += 1

	elif self.counter == 3:
		self.update_log1.set_description_str(msg)
		self.update_log2.set_description_str(self.old)
		self.update_log3.set_description_str(self.old_old)

		self.old_old = self.old
		self.old = msg					
		self.counter += 1

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
		tty.setraw(STDIN_FILENO)

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

	def __interact_copy(self, output_filter=None):
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
				data = os.read(STDIN_FILENO, 1000)
				while data != b'' and self.isalive():
					n = os.write(self.child_fd, data)
					data = data[n:]
