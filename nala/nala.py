from os import get_terminal_size, getuid, environ
from textwrap import TextWrapper
from datetime import datetime
from getpass import getuser
from pathlib import Path
from click import style
from sys import argv
import hashlib
import errno
from tqdm import tqdm
import json
import apt_pkg
from nala.columnar import Columnar
from nala.utils import dprint, iprint, logger_newline, ask, shell, RED, BLUE, YELLOW, GREEN
from nala.progress import nalaCache, nalaOpProgress, nalaProgress, InstallProgress

columnar = Columnar()
timezone = datetime.utcnow().astimezone().tzinfo
time = datetime.now().strftime('%Y-%m-%d %H:%M:%S')+' '+str(timezone)

try:
	columns = get_terminal_size().columns
except OSError:
	columns = 80

w_width = columns

if w_width > 86:
	w_width = 86

wrapper = TextWrapper(w_width, subsequent_indent='   ' )

try:
	USER = environ["SUDO_USER"]
	UID = environ["SUDO_UID"]
except KeyError:
	USER = getuser()
	UID = getuid()
	
environ["DEBIAN_FRONTEND"] = "noninteractive"


NALA_DIR = Path('/var/lib/nala')
NALA_HISTORY = Path('/var/lib/nala/history')

class nala:

	def __init__(self,  download_only = False,
						assume_yes = False,
						verbose = False,
						debug = False,
						no_update = False,
						metalink_out = None,
						hash_check = False,
						aria2c = '/usr/bin/aria2c'):

		if not no_update:
			print('Updating package list...')
			nalaCache().update(nalaProgress(verbose=verbose))

		self.cache = nalaCache(nalaOpProgress(verbose=verbose))
		self.download_only = download_only
		self.verbose = verbose
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

		# Lists to check if we're removing stuff we shouldn't
		self.essential = []
		self.nala = []
		self.nala_depends = ['nala', 'python3-pyshell']

	def upgrade(self, dist_upgrade=False):
		self.cache.upgrade(dist_upgrade=dist_upgrade)
		self.auto_remover()
		self._get_changes(upgrade=True)
	
	def install(self, pkg_names):
		dprint(f"Install pkg_names: {pkg_names}")
		not_found = []
		for pkg_name in pkg_names:
			if pkg_name in self.cache:
				pkg = self.cache[pkg_name]
				if not pkg.installed:
					pkg.mark_install()
					dprint(f"Marked Install: {pkg.name}")
				elif pkg.is_upgradable:
					pkg.mark_upgrade()
					dprint(f"Marked upgrade: {pkg.name}")
				elif not pkg.is_upgradable:
					print(f'Package {style(pkg.name, **GREEN)}',
					'is already at the latest version',
					style(pkg.installed.version, **BLUE))
			else:
				not_found.append(pkg_name)
		if not_found:
			for pkg in not_found:
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
		not_found = []
		not_installed = []

		for pkg_name in pkg_names:
			if pkg_name in self.cache:
				pkg = self.cache[pkg_name]
				if pkg.installed:
					if not pkg.essential:
						if not pkg.shortname in self.nala_depends:
							pkg.mark_delete()
							dprint(f"Marked delete: {pkg.name}")
						else:
							self.nala.append(pkg.name)
					else:
						self.essential.append(pkg.name)
				else:
					not_installed.append(pkg.name)
			else:
				not_found.append(pkg_name)

		if not_installed:
			if len(not_installed) > 4:
				print(
					style('Error:', **RED),
					"Packages",
					style(",".join(pkg for pkg in not_installed), **YELLOW),
					"not installed"
				)
			else:
				for pkg in not_installed:
					print(
						style('Error:', **RED),
						style(pkg, **YELLOW),
						'not installed'
					)

		if not_found:
			for pkg in not_found:
				print(
					style('Error:', **RED),
					style(pkg, **YELLOW),
					'not found'
				)

		if self.essential:
			for pkg in self.essential:
				print(
					style('Error:', **RED),
					style(pkg, **YELLOW),
					'cannot be removed'
				)

			print(f"Maybe {style('apt', **RED)} will let you")
			exit(1)

		if self.nala:
			for pkg in self.nala:
				print(
					style('Error:', **RED),
					style(pkg, **YELLOW),
					'cannot be removed'
				)

			print(f"Why do you think I would destroy myself?")
			exit(1)

		self.auto_remover()
		self._get_changes(remove=True)

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
			exit()
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
			print("History has been cleared")
			return

		elif isinstance(id, int):
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
		else:
			print("\nYour option was not understood")

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
		essential = []
		autoremove = []
		nala = []
		for pkg in self.cache:
			if pkg.is_auto_removable:
				if not pkg.essential:
					if not pkg.shortname in self.nala_depends:
						pkg.mark_delete()
						autoremove.append(
							f"<Package: '{pkg.name}' Arch: '{pkg.installed.architecture}' Version: '{pkg.installed.version}'"
						)
					else:
						nala.append(pkg.name)
				else:
					essential.append(pkg.name)

		if essential:
			for pkg in essential:
				print(
					style('Error:', **RED),
					style(pkg, **YELLOW),
					'cannot be removed'
				)

			print("What ever you did tried to auto mark essential packages")
			exit(1)

		if nala:
			for pkg in nala:
				print(
					style('Error:', **RED),
					style(pkg, **YELLOW),
					'cannot be removed'
				)
			print("What ever you did would have resulted in my own removal!")
			exit(1)

		dprint(f"Pkgs marked by autoremove: {autoremove}")

	def _get_changes(self, upgrade=False, remove=False):
		pkgs = sorted(self.cache.get_changes(), key=lambda p:p.name)
		if not NALA_DIR.exists():
			NALA_DIR.mkdir()

		if upgrade and not pkgs:
			print(style("All packages are up to date.", bold=True))
			exit(0)
		elif not remove and not pkgs:
			print(style("Nothing for Nala to do.", bold=True))
			exit(0)
		elif remove and not pkgs:
			print(style("Nothing for Nala to remove.", bold=True))
			exit(0)
		if pkgs:
			_print_update_summary(self.cache, pkgs)
			if not self.assume_yes:
				if not ask('Do you want to continue'):
					print("Abort.")
					return
			
			for pkg in pkgs:
				if pkg.essential and pkg.marked_delete:
					self.essential.append(pkg.name)

				elif pkg.shortname in self.nala_depends and pkg.marked_delete:
					self.nala.append(pkg.name)

			if self.essential:
				for pkg in self.essential:
					print(
						style('Error:', **RED),
						style(pkg, **YELLOW),
						'cannot be removed'
					)

				print(f"Maybe {style('apt', **RED)} will let you")
				exit(1)

			if self.nala:
				for pkg in self.nala:
					print(
						style('Error:', **RED),
						style(pkg, **YELLOW),
						'cannot be removed'
					)

				print("For some reason I was about to get nuked")
				exit(1)

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

		if self.download_only:
			print("Download complete and in download only mode.")
		else:
			# This is the correct posistion configure it
			# apt_pkg.config.set('Dpkg::Options::', '--force-confdef')
			# apt_pkg.config.set('Dpkg::Options::', '--force-confold')
			# apt_pkg.config.set('Dpkg::Options::', '--force-confnew')
			# apt_pkg.config.set('Dpkg::Options::', '--force-confmiss')

			if upgrade:
				self.cache.commit(
					nalaProgress(self.verbose, self.debug, quiet=True),
					InstallProgress(self.verbose, self.debug)
				)
			else:
				# Install and remove get handled differently because we want the update
				# To be silent because there isn't one really. since we use tqdm bars
				# For the information it leaves 3 blank lines that looks gross
				from nala.progress import base
				self.cache.commit(
					base.AcquireProgress(),
					InstallProgress(self.verbose, self.debug)
				)

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
		)
		make_metalink(proc.stdin, pkgs)
		proc.stdin.close()
		# for verbose mode we just print aria2c output normally
		if self.verbose or self.debug:
			for line in iter(proc.stdout.readline, ''):
				# We don't really want this double printing if we have
				# Verbose and debug enabled.
				if line != '':
					if self.debug:
						dprint(line)
					else:
						print(line)
		# In normal mode we hide the downloads behind some fancy bars
		else:
			num = 0
			total = len(pkgs)
			
			with tqdm(total=total,
				colour='CYAN',
				desc=style('Downloading Packages', **BLUE),
				unit='pkg',
				position=1,
				bar_format='{l_bar}{bar}| {n_fmt}/{total_fmt} [{rate_fmt}{postfix}]',

			) as progress:
				with tqdm(total=0, position=0, bar_format='{desc}') as pkg_log:
					for line in iter(proc.stdout.readline, ''):
						try:
							deb_name = Path(pkgs[num].candidate.filename).name
						except IndexError:
							pass
						pkg_log.set_description_str(f"{style('Current Package:', **GREEN)} {deb_name}")
						if 'Download complete:' in line:
							num = num + 1
							progress.update(1)
						if num > total:
							num = total	
		
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
