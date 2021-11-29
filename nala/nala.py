from os import get_terminal_size, getuid, environ
from textwrap import TextWrapper
from datetime import datetime
from getpass import getuser
from pathlib import Path
from click import style
from sys import argv
import hashlib
import errno
import json

import apt
import apt_pkg
from nala.columnar import Columnar
from nala.utils import dprint, iprint, logger_newline, ask, shell
from apt.progress.text import AcquireProgress as TextProgress, _
from apt.progress.base import AcquireProgress as BaseProgress

columnar = Columnar()
timezone = datetime.utcnow().astimezone().tzinfo
time = datetime.now().strftime('%Y-%m-%d %H:%M:%S')+' '+str(timezone)

width = get_terminal_size().columns
if width > 86:
	width = 86
wrapper = TextWrapper(width, subsequent_indent='   ' )

# Click Style Colors
RED = {'fg':'red', 'bold':True}
YELLOW = {'fg':'yellow', 'bold':True}
GREEN = {'fg':'green', 'bold':True}
BLUE = {'fg':'blue', 'bold':True}

try:
	USER = environ["SUDO_USER"]
	UID = environ["SUDO_UID"]
except KeyError:
	USER = getuser()
	UID = getuid()

NALA_DIR = Path('/var/lib/nala')
NALA_HISTORY = Path('/var/lib/nala/history')

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
					fetch_progress = BaseProgress()
				try:
					res = self._cache.update(fetch_progress, slist,
											 pulse_interval)
				except SystemError as e:
					raise FetchFailedException(e)
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
class nalaProgress(TextProgress):

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

	def ims_hit(self, item):
		# type: (apt_pkg.AcquireItemDesc) -> None
		"""Called when an item is update (e.g. not modified on the server)."""
		BaseProgress.ims_hit(self, item)
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
		# type: (apt_pkg.AcquireItemDesc) -> None
		"""Called when an item is failed."""
		BaseProgress.fail(self, item)
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
		BaseProgress.fetch(self, item)
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

	def stop(self):
		# type: () -> None
		"""Invoked when the Acquire process stops running."""
		try:
			BaseProgress.stop(self)
		except KeyboardInterrupt:
			exit()
		# Trick for getting a translation from apt
		fetched = apt_pkg.size_to_str(self.fetched_bytes)
		elapsed = apt_pkg.time_to_str(self.elapsed_time)
		speed = apt_pkg.size_to_str(self.current_cps).rstrip("\n")

		self._write(
			style(
				f"Fetched {fetched}B in {elapsed} ({speed}B/s)",
				bold=True
			)
		)
		dprint(f"Fetched {fetched}B in {elapsed} ({speed}B/s)")

		# Delete the signal again.
		import signal
		signal.signal(signal.SIGWINCH, self._signal)

class nala:

	def __init__(self,  download_only = False,
						assume_yes = False,
						debug = False,
						no_update = False,
						metalink_out = None,
						hash_check = False,
						aria2c = '/usr/bin/aria2c'):

		if not no_update:
			print('Updating package list...')
			nalaCache().update(nalaProgress())
		if debug:
			self.cache = apt.Cache(apt.progress.text.OpProgress())
		else:
			self.cache = apt.Cache()

		self.download_only = download_only
		self.debug = debug
		self.assume_yes = assume_yes
		self.metalink_out = metalink_out
		self.hash_check = hash_check
		self.aria2c = aria2c
		self.archive_dir = Path(apt_pkg.config.find_dir('Dir::Cache::Archives'))
		"""/var/cache/apt/archives/"""
		if not self.archive_dir:
			raise Exception(('No archive dir is set.'
							 ' Usually it is /var/cache/apt/archives/'))

	def upgrade(self, dist_upgrade=False):
		self.cache.upgrade(dist_upgrade=dist_upgrade)
		self.auto_remover()
		self._get_changes(upgrade=True)
	
	def install(self, pkg_names):
		dprint(f"Install pkg_names: {pkg_names}")
		for pkg_name in pkg_names:
			if pkg_name in self.cache:
				pkg = self.cache[pkg_name]
				if not pkg.installed:
					pkg.mark_install()
					dprint(f"Marked Install: {pkg.name}")
				elif pkg.is_upgradable:
					pkg.mark_upgrade()
					dprint(f"Marked upgrade: {pkg.name}")
			else:
				print(
					style('Error:', **RED),
					style(pkg_name, **YELLOW),
					'not found'
				)
				exit(1)
		
		self.auto_remover()
		self._get_changes()

	def remove(self, pkg_names):
		dprint(f"Remove pkg_names: {pkg_names}")
		for pkg_name in pkg_names:
			if pkg_name in self.cache:
				pkg = self.cache[pkg_name]
				if pkg.installed:
					pkg.mark_delete()
					dprint(f"Marked delete: {pkg.name}")
			else:
				print(
					style('Error:', **RED),
					style(pkg_name, **YELLOW),
					'not found'
				)
				exit(1)

		self.auto_remover()
		self._get_changes()

	def show(self, pkg_names):
		dprint(f"Show pkg_names: {pkg_names}")
		print()
		for pkg_name in pkg_names:
			if pkg_name in self.cache:
				pkg = self.cache[pkg_name]
				origin = pkg.candidate.origins[0]
				arch = pkg.candidate.architecture
				print(f"Package: {pkg.name}")
				print(f"Version: {pkg.candidate.version}")
				print(f"Architecture: {arch}")
				print(f"Priority: {pkg.candidate.priority}")
				print(f"Section: {pkg.candidate.section}")
				print(f"Source: {pkg.candidate.source_name}")
				print(f"Origin: {pkg.candidate.origins[0].origin}")
				print(f"Maintainer: {pkg.candidate.record.get('Maintainer')}")
				if pkg.candidate.record.get('Original-Maintainer'):
					print(f"Original-Maintainer: {pkg.candidate.record.get('Original-Maintainer')}")
				print(f"Bugs: {pkg.candidate.record.get('Bugs')}")
				print(f"Installed-Size: {unit_str(pkg.candidate.installed_size, 1)}")

				if pkg.candidate.dependencies:
					print(style('Depends:', bold=True))
					for dep in pkg.candidate.dependencies:
						print(style('  '+dep.rawstr, fg='green', bold=True))

				if pkg.candidate.recommends:
					print(style('Recommends:', bold=True))
					for rec in pkg.candidate.recommends:
						print(style('  '+rec.rawstr, fg='green', bold=True))

				if pkg.candidate.suggests:
					print(style('Suggests:', bold=True))
					for sug in pkg.candidate.suggests:
						print(style('  '+sug.rawstr, fg='green', bold=True))

				print(f"Homepage: {pkg.candidate.homepage}")
				print(f"Download-Size: {unit_str(pkg.candidate.size, 1)}")

				if origin.archive == 'now':
					print('APT-Sources: /var/lib/dpkg/status')
				else:
					print(
						f"APT-Sources: http://{origin.site}/{origin.origin.lower()}",
						f"{origin.archive}/{origin.component} {arch} Packages"
					)

				print(f"Description: {pkg.candidate._translated_records.long_desc}")
				print()
			else:
				print(
					style('Error:', **RED),
					style(pkg_name, **YELLOW),
					'not found'
				)
				exit(1)

	def history(self):
		if not NALA_HISTORY.exists():
			print("No history exists..")
			return
		history = NALA_HISTORY.read_text().splitlines()
		headers = ['ID:', 'Command:', 'Date and Time:', 'Altered:']
		names = []

		for transaction in history:
			trans = []
			transaction = json.loads(transaction)
			trans.append(transaction.get('ID'))
			trans.append(' '.join(com for com in transaction.get('Command')))
			trans.append(transaction.get('Date'))
			trans.append(transaction.get('Altered'))
			names.append(trans)
		print(columnar(names, headers, no_borders=True))

	def _get_history(self, id):
		dprint(f"Getting history {id}")
		if not NALA_HISTORY.exists():
			print("No history exists..")
			return
		history = NALA_HISTORY.read_text().splitlines()
		TRANSACTION = {}

		for transaction in history:
			transaction = json.loads(transaction)
			if transaction.get('ID') == id:
				TRANSACTION = transaction
		return TRANSACTION

	def history_undo(self, id, redo: bool = False):
		if redo:
			dprint(f"History: redo {id}")
		else:
			dprint(f"History: undo {id}")
		transaction = self._get_history(id)

		dprint(f"Transaction: {transaction}")

		# We just reverse whatever was done in the transaction
		if transaction.get('Command')[0] == 'remove':
			removed_pkgs = transaction.get('Removed')
			if redo:
				self.remove(removed_pkgs)
			else:
				self.install(removed_pkgs)

		elif transaction.get('Command')[0] == 'install':
			installed_pkgs = transaction.get('Installed')
			if redo:
				self.install(installed_pkgs)
			else:
				self.remove(installed_pkgs)
		else:
			print('\nUndo for operations other than install or remove are not currently supported')

	def history_clear(self, id):
		dprint(f"History clear {id}")
		if not NALA_HISTORY.exists():
			print("No history exists to clear..")
			return
		
		if id == 'all':
			NALA_HISTORY.unlink()
			return

		history = NALA_HISTORY.read_text().splitlines()
		history_edit = []
		sum = 0
		# Using sum increments to relabled the IDs so when you remove just one
		# There isn't a gap in ID numbers and it looks concurrent.
		for transaction in history:
			transaction = json.loads(transaction)
			if transaction.get('ID') != id:
				sum = sum +1
				transaction['ID'] = sum
				history_edit.append(json.dumps(transaction))
		# Write the new history file	
		with open(NALA_HISTORY, 'w') as file:
			for line in history_edit:
				file.write(str(line)+'\n')

	def history_info(self, id):
		dprint(f"History info {id}")

		transaction = self._get_history(id)

		dprint(f"Transaction {transaction}")

		delete_names = transaction.get('Removed')
		install_names = transaction.get('Installed')
		upgrade_names = transaction.get('Upgraded')

		delete_info = []
		install_info = []
		upgrade_info = []

		for pkg in install_names:
			if pkg in self.cache:
				pkg = self.cache[pkg]
				install_info.append(
					[style(pkg.name, **GREEN),
					pkg.candidate.version,
					unit_str(pkg.candidate.size)],
				)
		for pkg in delete_names:
			if pkg in self.cache:
				pkg = self.cache[pkg]
				delete_info.append(
					[style(pkg.name, **RED),
					pkg.candidate.version,
					unit_str(pkg.candidate.size)],
				)
		for pkg in upgrade_names:
			if pkg in self.cache:
				pkg = self.cache[pkg]
				upgrade_info.append(
					[style(pkg.name, **BLUE),
					pkg.candidate.version,
					unit_str(pkg.candidate.size)],
				)

		pprint_names(['Package:', 'Version:', 'Size:'], delete_info, 'Removed:')
		pprint_names(['Package:', 'Version:', 'Size:'], install_info, 'Installed:')
		pprint_names(['Package:', 'Version:', 'Size:'], upgrade_info, 'Upgraded:')

	def auto_remover(self):
		autoremove = []
		for pkg in self.cache:
			if pkg.is_auto_removable:
				pkg.mark_delete()
				autoremove.append(
					f"<Package: '{pkg.name}' Arch: '{pkg.installed.architecture}' Version: '{pkg.installed.version}'"
				)
		dprint(f"Pkgs marked by autoremove: {autoremove}")

	def _get_changes(self, upgrade=False):
		pkgs = sorted(self.cache.get_changes(), key=lambda p:p.name)
		if not NALA_DIR.exists():
			NALA_DIR.mkdir()
	
		if pkgs:
			_print_update_summary(self.cache, pkgs)
			if not self.assume_yes:
				if not ask('Do you want to continue'):
					print("Abort.")
					return
			
			_write_history(pkgs)
			_write_log(pkgs)

			pkgs = [pkg for pkg in pkgs if not pkg.marked_delete and \
						not self._file_downloaded(pkg, hash_check = \
													  self.hash_check)]
			if self.metalink_out:
				with open(self.metalink_out, 'w', encoding='utf-8') as f:
					make_metalink(f, pkgs)
				return
			if not self._download(pkgs, num_concurrent=guess_concurrent(pkgs)):
				print("Some downloads failed. apt_pkg will take care of them.")
		if upgrade and not pkgs:
			print("All packages are up to date.")
			exit(0)
		if self.download_only:
			print("Download complete and in download only mode.")
		else:
			self.cache.commit(apt.progress.text.AcquireProgress())

	def _download(self, pkgs, num_concurrent=1):
		if not pkgs:
			return True
		partial_dir = self.archive_dir / 'partial'
		cmdline = [self.aria2c,
				   '--metalink-file=-',
				   '--file-allocation=none',
				   '--auto-file-renaming=false',
				   f'--dir={partial_dir}',
				   f'--max-concurrent-downloads={num_concurrent}',
				   '--no-conf',
				   '--remote-time=true',
				   '--auto-save-interval=0',
				   '--continue',
				   '--split=1'
				   ]
		if self.hash_check:
			cmdline.append('--check-integrity=true')

		http_proxy = apt_pkg.config.find('Acquire::http::Proxy')
		https_proxy = apt_pkg.config.find('Acquire::https::Proxy', http_proxy)
		ftp_proxy = apt_pkg.config.find('Acquire::ftp::Proxy')

		if http_proxy:
			cmdline.append(f'--http-proxy={http_proxy}')
		if https_proxy:
			cmdline.append(f'--https-proxy={https_proxy}')
		if ftp_proxy:
			cmdline.append(f'--ftp-proxy={ftp_proxy}')

		proc = shell(
			cmdline,
			popen=True,
			stdin=shell.PIPE,
			capture_output=shell.DEFAULT
		)
		make_metalink(proc.stdin, pkgs)
		proc.stdin.close()
		proc.wait()
		link_success = True
		# Link archives/partial/*.deb to archives/
		for pkg in pkgs:
			filename = get_filename(pkg.candidate)
			dst = self.archive_dir / filename
			src = partial_dir / filename
			ctrl_file = Path(str(src) + '.aria2')

			# If control file exists, we assume download is not
			# complete.
			if ctrl_file.exists():
				continue
			try:
				# Making hard link because aria2c needs file in
				# partial directory to know download is complete
				# in the next invocation.
				src.rename(dst)
			except OSError as e:
				if e.errno != errno.ENOENT:
					print("Failed to move archive file", e)
				link_success = False
		return proc.returncode == 0 and link_success

	def _file_downloaded(self, pkg, hash_check=False):
		candidate = pkg.candidate
		path = self.archive_dir / get_filename(candidate)
		if not path.exists() or path.stat().st_size != candidate.size:
			return False
		if hash_check:
			hash_type, hash_value = get_hash(pkg.candidate)
			try:
				return check_hash(path, hash_type, hash_value)
			except IOError as e:
				if e.errno != errno.ENOENT:
					print("Failed to check hash", e)
				return False
		else:
			return True

def check_hash(path, hash_type, hash_value):
	hash_fun = hashlib.new(hash_type)
	with open(path) as f:
		while 1:
			bytes = f.read(4096)
			if not bytes:
				break
			hash_fun.update(bytes)
	return hash_fun.hexdigest() == hash_value

def get_hash(version):
	if version.sha256:
		return ("sha256", version.sha256)
	elif version.sha1:
		return ("sha1", version.sha1)
	elif version.md5:
		return ("md5", version.md5)
	else:
		return (None, None)

def get_filename(version):
	# TODO apt-get man page said filename and basename in URI
	# could be different.
	return Path(version.filename).name

def make_metalink(out, pkgs):
	out.write('<?xml version="1.0" encoding="UTF-8"?>')
	out.write('<metalink xmlns="urn:ietf:params:xml:ns:metalink">')
	for pkg in pkgs:
		version = pkg.candidate
		hashtype, hashvalue = get_hash(version)
		out.write('<file name="{0}">'.format(get_filename(version)))
		out.write('<size>{0}</size>'.format(version.size))
		if hashtype:
			out.write('<hash type="{0}">{1}</hash>'.format(hashtype, hashvalue))
		for uri in version.uris:
			out.write('<url priority="1">{0}</url>'.format(uri))
		out.write('</file>')
	out.write('</metalink>')

def guess_concurrent(pkgs):
	max_uris = 0
	for pkg in pkgs:
		version = pkg.candidate
		max_uris = max(len(version.uris), max_uris)
	if max_uris == 1:
		max_uris = 2
	return max_uris

def pprint_names(headers, names, title):
	columns, lines = get_terminal_size()
	if names:
		print('='*columns)
		print(title)
		print('='*columns, end='')
		print(columnar(names, headers, no_borders=True, justify=['l', 'l', 'r']))

def transaction_summary(names, width, header: str):
	if names:
		print(
			header.ljust(7),
			f'{len(names)}'.rjust(width),
			'Packages'
			)

def unit_str(val, just = 7):
	if val > 1000*1000:
		return f"{val/1000/1000 :.1f}".rjust(just)+" MB"
	elif val > 1000:
		return f"{round(val/1000) :.0f}".rjust(just)+" kB"
	else:
		return f'{val :.0f}'.rjust(just)+" B"

def _write_history(pkgs):

	delete_names = []
	install_names = []
	upgrade_names = []

	# TODO marked_downgrade, marked_keep, marked_reinstall
	for pkg in pkgs:
		if pkg.marked_delete:
			delete_names.append(pkg.name)

		elif pkg.marked_install:
			install_names.append(pkg.name)

		elif pkg.marked_upgrade:
			upgrade_names.append(pkg.name)

	history = []
	if NALA_HISTORY.exists():
		history = NALA_HISTORY.read_text().splitlines()

	if history:
		ID = len(history) + 1
	else:
		ID = 1
	
	altered = len(delete_names) + len(install_names) + len(upgrade_names)

	transaction = {
		'ID' : ID,
		'Date' : time,
		'Command' : argv[1:],
		'Altered' : altered,
		'Removed' : delete_names,
		'Installed' : install_names,
		'Upgraded' : upgrade_names,
	}

	history.append(json.dumps(transaction))

	with open(NALA_HISTORY, 'w') as file:
		for line in history:
			file.write(str(line)+'\n')

def _write_log(pkgs):
	delete_names = []
	install_names = []
	upgrade_names = []

	# TODO marked_downgrade, marked_keep, marked_reinstall
	for pkg in pkgs:
		if pkg.marked_delete:
			delete_names.append(f"{pkg.name}:{pkg.installed.architecture} ({pkg.installed.version})")

		elif pkg.marked_install:
			install_names.append(f"{pkg.name}:{pkg.candidate.architecture} ({pkg.candidate.version})")

		elif pkg.marked_upgrade:
			upgrade_names.append(f"{pkg.name}:{pkg.candidate.architecture} ({pkg.candidate.version})")

	# Just logfile things
	iprint(f'Date: {time}')
	iprint(f'Requested-By: {USER} ({UID})')

	if delete_names:
		iprint(f'Removed: {", ".join(item for item in delete_names)}')
	if install_names:
		iprint(f'Installed: {", ".join(item for item in install_names)}')
	if upgrade_names:
		iprint(f'Upgraded: {", ".join(item for item in upgrade_names)}')

	logger_newline()
	
def _print_update_summary(cache, pkgs):
	columns, lines = get_terminal_size()

	delete_names = []
	install_names = []
	upgrade_names = []

	# TODO marked_downgrade, marked_keep, marked_reinstall
	for pkg in pkgs:
		if pkg.marked_delete:
			delete_names.append(
				[style(pkg.name, **RED),
				pkg.installed.version,
				unit_str(pkg.installed.size)]
			)

		elif pkg.marked_install:
			install_names.append(
				[style(pkg.name, **GREEN),
				pkg.candidate.version,
				unit_str(pkg.candidate.size)],
			)

		elif pkg.marked_upgrade:
			upgrade_names.append(
				[style(pkg.name, **BLUE),
				pkg.candidate.version,
				unit_str(pkg.candidate.size)]
			)

	pprint_names(['Package:', 'Version:', 'Size:'], delete_names, 'Removing:')
	pprint_names(['Package:', 'Version:', 'Size:'], install_names, 'Installing:')
	pprint_names(['Package:', 'Version:', 'Size:'], upgrade_names, 'Upgrading:')

	# We need to get our width for formating
	width_list = [
		len(delete_names),
		len(install_names),
		len(upgrade_names)
	]

	# I know this looks weird but it's how it has to be
	width = len(str(max(width_list)))

	print('='*columns)
	print('Summary')
	print('='*columns)
	transaction_summary(install_names, width, 'Install')
	transaction_summary(upgrade_names, width, 'Upgrade')
	transaction_summary(delete_names, width, 'Remove')
	
	if cache.required_download > 0:
		print(f'\nTotal download size: {unit_str(cache.required_download)}')
	else:
		# We need this extra line lol
		print()
	if cache.required_space < 0:
		print(f'Disk space to free: {unit_str(-cache.required_space)}')
	else:
		print(f'Disk space required: {unit_str(cache.required_space)}')
