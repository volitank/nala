import os
import sys
import re
import fcntl
import errno
import select
import signal

import apt
import apt_pkg
from tqdm import tqdm
from pathlib import Path
from pexpect.fdpexpect import fdspawn
import apt.progress.base as base
import apt.progress.text as text
from click import style

from nala.utils import RED, BLUE, GREEN, YELLOW, ask, dprint

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
class nalaProgress(text.AcquireProgress):

	def __init__(	self, verbose=False, debug=False, outfile=None):
		text.TextProgress.__init__(self, outfile)
		base.AcquireProgress.__init__(self)

		self._file = outfile or sys.stdout

		self._signal = None
		self._width = 80
		self._id = 1
		self.counter = 1
		self.verbose=verbose
		self.debug=debug

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
				#self._file.write("\r")
				self._file.flush()
		else:
			for item in ['Updated:', 'Ignored:', 'Error:', 'No Change:']:
				if item in msg:
					if self.counter == 4:
						self.counter = 3

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
					break
			else:
				self.progress.set_description_str(f"{style(msg.strip(), bold=True)}")

	def ims_hit(self, item):
		"""Called when an item is update (e.g. not modified on the server)."""
		base.AcquireProgress.ims_hit(self, item)
		no_change = style('No Change:', **GREEN)
		line = f"{no_change} {item.description}"
		dline = f"'No Change:' {item.description}"
		if item.owner.filesize:
			size = apt_pkg.size_to_str(item.owner.filesize)
			line += f' [{size}B]'
			dline += f' [{size}B]'
		self._write(line)
		dprint(dline)

	def fail(self, item):
		"""Called when an item is failed."""
		base.AcquireProgress.fail(self, item)
		if item.owner.status == item.owner.STAT_DONE:
			ignored = style('Ignored:  ', **YELLOW)
			self._write(f"{ignored} {item.description}")
			dprint(f"'Ignored:  '{item.description}")
		else:
			err = style('Error:    ', **RED)
			self._write(f"{err} {item.description}")
			self._write(f"  {item.owner.error_text}")
			dprint(f"Error:     {item.description}")
			dprint(f"  {item.owner.error_text}")

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
		dprint(f"Updated:   {item.description}")

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
		self.progress = tqdm(position=3, bar_format='{desc}')
		self.update_log1 = tqdm(position=2, bar_format='{desc}')
		self.update_log2 = tqdm(position=1, bar_format='{desc}')
		self.update_log3 = tqdm(position=0, bar_format='{desc}')

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

		if self.verbose:
			self._write(
				style(msg, bold=True)
			)
		else:
			self.progress.set_description_str(f"{style(msg, bold=True)}")

		dprint(msg)
		# Delete the signal again.
		signal.signal(signal.SIGWINCH, self._signal)

		self.update_log3.close()
		self.update_log2.close()
		self.update_log1.close()
		self.progress.close()
			
class nalaOpProgress(base.OpProgress, nalaProgress):
	"""Operation progress reporting.

	This closely resembles Optext.AcquireProgress in libapt-pkg.
	"""

	def __init__(self, outfile=None, verbose=False):
		nalaProgress.__init__(self, outfile, verbose)
		base.OpProgress.__init__(self)
		self.old_op = ""
		self.verbose = verbose
		self.counter = 1

	def update(self, percent=None):
		"""Called periodically to update the user interface."""
		base.OpProgress.update(self, percent)
		if self.verbose:
			if self.major_change and self.old_op:
				self._write(self.old_op)	
			self._write("%s... %i%%\r" % (self.op, self.percent), False, True)
			self.old_op = self.op

	def done(self):
		"""Called once an operation has been completed."""
		base.OpProgress.done(self)
		if self.verbose:
			if self.old_op:
				self._write(text._("%c%s... Done") % ('\r', self.old_op), True, True)
			self.old_op = ""

class InstallProgress(base.InstallProgress):

	def __init__(self, verbose: bool = False,
						debug: bool = False):
		self.verbose = verbose
		self.debug = debug
		self.counter = 1

		(self.statusfd, self.writefd) = os.pipe()
		# These will leak fds, but fixing this safely requires API changes.
		self.write_stream = os.fdopen(self.writefd, "w")
		self.status_stream = os.fdopen(self.statusfd, "r")
		fcntl.fcntl(self.statusfd, fcntl.F_SETFL, os.O_NONBLOCK)

	def start_update(self):
		"""(Abstract) Start update."""
		self.extra_info = []
		self.notice = []
		# Create pkg bars
		self.pkg_log1 = tqdm(position=2, bar_format='{desc}')
		self.pkg_log2 = tqdm(position=1, bar_format='{desc}')
		self.pkg_log3 = tqdm(position=0, bar_format='{desc}')

	def finish_update(self):
		# type: () -> None
		"""(Abstract) Called when update has finished."""
		self.pkg_log3.close()
		self.pkg_log2.close()
		self.pkg_log1.close()
		if self.notice:
			for notice in self.notice:
				print(notice)
		print(style("Finished Successfully", **GREEN))

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
		
		pid, self.fd = self.fork()

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

		self.dpkg = os.fdopen(self.fd, "r")
		fcntl.fcntl(self.fd, fcntl.F_SETFL, os.O_NONBLOCK)

		# We use fdspawn from pexpect to interact with out dpkg pty
		self.child = fdspawn(self.dpkg)

		self.child_pid = pid
		res = self.wait_child()
		return os.WEXITSTATUS(res)

	def fork(self):
		"""Fork."""
		return os.forkpty()

	def format_dpkg_output(self, line):
		msg = ''
		for word in line:
			match = re.fullmatch('\(.*.\)', word)
			if word == 'Removing':
				msg = msg + style('Removing:   ', **RED)
			elif word == 'Unpacking':
				msg = msg + style('Unpacking:  ', **GREEN)
			elif word == 'Setting':
				msg = msg + style('Setting ', **GREEN)
			elif word == 'up':
				msg = msg + style('up: ', **GREEN)
			elif word == 'Processing':
				msg = msg + style('Processing: ', **GREEN)
			elif word == '...':
				continue
			elif match:
				word = re.sub('[()]', '', word)
				paren = style('(', bold=True)
				paren2 = style(')', bold=True)
				msg = msg + (' ') + paren+style(word, **BLUE)+paren2
			else:
				msg = msg + ' ' + word

		if self.verbose:
			print(msg.strip())
		else:
			# This whole thing is setup like this to give the scrolling effect on our 3 bars
			if self.counter == 4:
				self.counter = 3

			if self.counter == 1:
				self.old = msg
				self.pkg_log1.set_description_str(self.old)
				self.counter += 1

			elif self.counter == 2:
				self.pkg_log1.set_description_str(msg)
				self.pkg_log2.set_description_str(self.old)

				self.old_old = self.old
				self.old = msg
				self.counter += 1

			elif self.counter == 3:
				self.pkg_log1.set_description_str(msg)
				self.pkg_log2.set_description_str(self.old)
				self.pkg_log3.set_description_str(self.old_old)

				self.old_old = self.old
				self.old = msg					
				self.counter += 1
	
	def update_interface(self):
		"""Update the interface."""
		try:
			dpkg_pty = self.dpkg.readline().strip()
		except IOError as err:
			# resource temporarly unavailable is ignored
			if err.errno != errno.EAGAIN and err.errno != errno.EWOULDBLOCK:
				if err.strerror != 'Input/output error':
					print('err:', err.strerror)
			return

		try:
			# Everything dealing with line is mostly just for the conffile
			# Maybe we use it for error handling in the future as well.
			line = self.status_stream.readline()
		except IOError as err:
			# resource temporarly unavailable is ignored
			if err.errno != errno.EAGAIN and err.errno != errno.EWOULDBLOCK:
				print(err.strerror)
			return

		pkgname = status = status_str = percent = base = ""

		if line.startswith('pm'):
			try:
				(status, pkgname, percent, status_str) = line.split(":", 3)
			except ValueError:
				# silently ignore lines that can't be parsed
				return

		# Always strip the status message
		pkgname = pkgname.strip()
		status_str = status_str.strip()
		status = status.strip()

		if status == 'pmerror' or status == 'error':
			self.error(pkgname, status_str)
		elif status == 'conffile-prompt' or status == 'pmconffile':
		# Looks like pmconffile:/etc/postfixadmin/config.inc.php:40.0000:'/etc/postfixadmin/config.inc.php' '/etc/postfixadmin/config.inc.php.dpkg-new' 1 1
			match = re.match("\\s*\'(.*)\'\\s*\'(.*)\'.*", status_str)
			if match:
				self.conffile(match.group(1), match.group(2))
				self.pkg_log3.clear()
				self.pkg_log2.clear()
				self.pkg_log1.clear()
				print(style('Dpkg has an update for:', **YELLOW), match.group(1))
				print(style('It seems this config has been modified locally.', **YELLOW))
				#print('new', match.group(2))
				if ask(style('Would you like to replace your modified config with a new version', bold=True), default_no=True):
					# Replace config
					self.child.write('y\r')
				else:
					# Keep old config
					self.child.write('n\r')

		conf_file = [
			"Configuration file",
			"==> Modified (by you or by a script) since installation.",
			"==> Package distributor has shipped an updated version.",
			"What would you like to do about it ?  Your options are:",
			"Y or I  : install the package maintainer's version",
			"N or O  : keep your currently-installed version",
			"D     : show the differences between the versions",
			"Z     : start a shell to examine the situation",
			"The default action is to keep your current version.",
			"*** config.inc.php (Y/I/N/O/D/Z) [default=N] ?",
		]

		extra = [
			# Stuff that's pretty useless
			'(Reading database', #'(Reading database ... 247588 files and directories currently installed.)'
			# 'Input/output', # 'Input/output error'
			'Selecting previously unselected package', # 'Selecting previously unselected package chafa.'
			'Preparing to unpack', # 'Preparing to unpack .../2-chafa_1.8.0-1_amd64.deb ...'
			'Extracting templates from packages:',
			'Preconfiguring packages',
			'Reloading AppArmor profiles',

			# Mono Stuff
			'Mono',
			'Motus Technologies',
			'Importing into',
			'I already trust',
			'updates of cacerts keystore disabled',
			'Import process completed',

			# Stuff that could be useful
			'disabled or a static unit not running',
			'Sucessfully set capabilities',
			"Leaving 'diversion of",
			'Installing new version of config file',
			'update-alternatives:',
			'update-initramfs:',
			'Installation finished. No error reported.',
			'Generating grub configuration file',
			'Found linux image:',
			'Found initrd image:',
			'Warning: os-prober will not be executed to detect other bootable partitions.',
			'Systems on them will not be added to the GRUB boot configuration.',
			'Check GRUB_DISABLE_OS_PROBER documentation entry.',
			'Adding boot menu entry for UEFI Firmware Settings',
			'No such file or directory',
			'dpkg: warning: while removing',
			'rmdir: failed to remove',
			'Running hooks in',
			'Updating certificates',
			'Running hooks',
			'Done',
			'done',
			'Mono',
		]

		KERNEL_MSG = 'The currently running kernel version is not the expected kernel version'
		REBOOT_MSG = 'so you should consider rebooting.'

		if dpkg_pty != '':

			if KERNEL_MSG in dpkg_pty:
				self.notice.append(dpkg_pty)

			if 'Please remove.' in dpkg_pty:
				self.notice.append(dpkg_pty)

			if REBOOT_MSG in dpkg_pty:
				self.notice.append(dpkg_pty)
				self.child.write('\r')

			if self.verbose:
				self.format_dpkg_output(dpkg_pty.split())

			elif self.debug:
				dprint(dpkg_pty)

			else:
				for item in extra:
					if item in dpkg_pty:
						break
					elif re.match('Installing for *.* platform', item):
						break
				else:
					self.format_dpkg_output(dpkg_pty.split())

	def wait_child(self):
		"""Wait for child progress to exit.

		This method is responsible for calling update_interface() from time to
		time. It exits once the child has exited. The return values is the
		full status returned from os.waitpid() (not only the return code).
		"""
		(pid, res) = (0, 0)
		while True:
			try:
				select.select([self.dpkg], [], [],
							  self.select_timeout)
			except select.error as error:
				(errno_, _errstr) = error.args
				if errno_ != errno.EINTR:
					raise

			self.update_interface()
			try:
				(pid, res) = os.waitpid(self.child_pid, os.WNOHANG)
				if pid == self.child_pid:
					break
			except OSError as err:
				if err.errno == errno.ECHILD:
					break
				if err.errno != errno.EINTR:
					raise

		return res
