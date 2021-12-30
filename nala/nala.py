from os import get_terminal_size, getuid, environ
from textwrap import TextWrapper
from datetime import datetime
from getpass import getuser
from pathlib import Path
from click import style
import fnmatch
from sys import argv, stderr, exit
import hashlib
import errno
import os
import json
import apt_pkg
from apt.cache import LockFailedException, Cache
from apt.package import Package
from subprocess import Popen
import requests

from nala.columnar import Columnar
from nala.utils import dprint, iprint, logger_newline, ask, shell, RED, BLUE, YELLOW, GREEN
from nala.dpkg import nalaProgress, InstallProgress
from nala.rich_custom import pkg_download_progress, rich_live, rich_grid
from nala.options import arguments

columnar = Columnar()
timezone = datetime.utcnow().astimezone().tzinfo
time = datetime.now().strftime('%Y-%m-%d %H:%M:%S')+' '+str(timezone)

try:
	columns = get_terminal_size().columns
except OSError:
	columns = 80

w_width = min(columns, 86)
wrapper = TextWrapper(w_width, subsequent_indent='   ')

try:
	USER = environ["SUDO_USER"]
	UID = environ["SUDO_UID"]
except KeyError:
	USER = getuser()
	UID = getuid()

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
						raw_dpkg = False,
						aria2c = '/usr/bin/aria2c'):

		# We want to update the cache before we initialize it
		if not no_update:
			print('Updating package list...')
			try:
				Cache().update(nalaProgress(verbose=verbose))
			except LockFailedException as e:
				print(f'{style("Error:", **RED)} {e}')
				print('Are you root?')
				exit()

		# We check the arguments here to see if we have any kind of
		# Non interactiveness going on
		self.noninteractive = arguments.noninteractive
		self.noninteractive_full = arguments.noninteractive_full
		self.confold = arguments.confold
		self.confnew = arguments.confnew
		self.confdef = arguments.confdef
		self.confmiss = arguments.confask
		self.confask = arguments.confask
		self.no_aptlist = arguments.no_aptlist

		try:
			self.cache = Cache(nalaProgress(verbose=verbose))
		except LockFailedException as e:
			print(f'{style("Error:", **RED)} {e}')
			print('Are you root?')
			exit()
		
		self.download_only = download_only
		self.verbose = verbose
		self.debug = debug
		self.assume_yes = assume_yes
		self.metalink_out = metalink_out
		self.hash_check = hash_check
		self.aria2c = aria2c
		self.raw_dpkg = raw_dpkg

		# This is just a flag to check if we downloaded anything
		self.downloaded = False
		self.purge = False

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
		self.get_changes(upgrade=True)

	def glob_filter(self, pkg_names: list):
		packages = self.cache.keys()
		new_packages = []

		for pkg_name in pkg_names:
			if '*' in pkg_name:
				dprint(f'Globbing: {pkg_name}')
				new_packages.extend(
					fnmatch.filter(packages, pkg_name)
				)
			else:
				new_packages.append(pkg_name)

		dprint(f'List after globbing: {new_packages}')
		return new_packages

	def install(self, pkg_names):
		dprint(f"Install pkg_names: {pkg_names}")
		not_found = []

		# We only want to glob if we detect an *
		if '*' in str(pkg_names):
			pkg_names = self.glob_filter(pkg_names)

		for pkg_name in pkg_names:
			if pkg_name not in self.cache:
				not_found.append(pkg_name)
			else:
				pkg = self.cache[pkg_name]
				if not pkg.installed:
					pkg.mark_install()
					dprint(f"Marked Install: {pkg.name}")
				elif pkg.is_upgradable:
					pkg.mark_upgrade()
					dprint(f"Marked upgrade: {pkg.name}")
				else:
					print(f'Package {style(pkg.name, **GREEN)}',
					'is already at the latest version',
					style(pkg.installed.version, **BLUE))

		if not_found:
			pkg_error(not_found, 'not found', terminate=True)
		
		self.auto_remover()
		self.get_changes()

	def remove(self, pkg_names, purge=False):
		dprint(f"Remove pkg_names: {pkg_names}")
		not_found = []
		not_installed = []

		self.purge = purge

		# We only want to glob if we detect an *
		if '*' in str(pkg_names):
			pkg_names = self.glob_filter(pkg_names)

		for pkg_name in pkg_names:
			if pkg_name not in self.cache:
				not_found.append(pkg_name)
				continue

			pkg = self.cache[pkg_name]
			if not pkg.installed:
				not_installed.append(pkg_name)
				continue

			pkg.mark_delete(purge=purge)
			dprint(f"Marked delete: {pkg.name}")

		if not_installed:
			pkg_error(not_installed, 'not installed')
		if not_found:
			pkg_error(not_found, 'not found')

		self.auto_remover()
		self.get_changes(remove=True)

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

				installed = 'yes' if pkg.is_installed else 'no'
				print(f"Installed: {installed}")
				print(f"Priority: {pkg.candidate.priority}")
				if pkg.essential:
					print('Essential: yes')
				print(f"Section: {pkg.candidate.section}")
				print(f"Source: {pkg.candidate.source_name}")
				print(f"Origin: {pkg.candidate.origins[0].origin}")
				print(f"Maintainer: {pkg.candidate.record.get('Maintainer')}")
				if pkg.candidate.record.get('Original-Maintainer'):
					print(f"Original-Maintainer: {pkg.candidate.record.get('Original-Maintainer')}")
				print(f"Bugs: {pkg.candidate.record.get('Bugs')}")
				print(f"Installed-Size: {unit_str(pkg.candidate.installed_size, 1)}")
				if pkg.candidate.provides:
					provides = pkg.candidate.provides
					provides.sort()
					print('Provides:', ", ".join(provides))

				if pkg.candidate.dependencies:
					print(style('Depends:', bold=True))
					dep_format(pkg.candidate.dependencies)

				if pkg.candidate.recommends:
					print(style('Recommends:', bold=True))
					dep_format(pkg.candidate.recommends)

				if pkg.candidate.suggests:
					print(style('Suggests:', bold=True))
					dep_format(pkg.candidate.suggests)

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

			command = transaction.get('Command')
			if command[0] in ['update', 'upgrade']:
				for package in transaction.get('Upgraded'):
					command.append(package[0])

			trans.append(' '.join(transaction.get('Command')))
			trans.append(transaction.get('Date'))
			trans.append(transaction.get('Altered'))
			names.append(trans)
		print(columnar(names, headers, no_borders=True, wrap_max=0))

	def get_history(self, id):
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
		transaction = self.get_history(id)

		dprint(f"Transaction: {transaction}")

		# We just reverse whatever was done in the transaction
		if transaction.get('Command')[0] == 'remove':
			removed_pkgs = (pkg[0] for pkg in transaction.get('Removed'))
			if redo:
				self.remove(removed_pkgs)
			else:
				self.install(removed_pkgs)

		elif transaction.get('Command')[0] == 'install':
			installed_pkgs = (pkg[0] for pkg in transaction.get('Installed'))
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
					sum += 1
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

		transaction = self.get_history(id)

		dprint(f"Transaction {transaction}")

		delete_names = transaction.get('Removed')
		install_names = transaction.get('Installed')
		upgrade_names = transaction.get('Upgraded')

		delete_info = []
		install_info = []
		upgrade_info = []

		for pkg in install_names:
			name, version, size = pkg
			install_info.append(
				[style(name, **GREEN),
				version,
				unit_str(size)],
			)

		for pkg in delete_names:
			name, version, size = pkg
			delete_info.append(
				[style(name, **RED),
				version,
				unit_str(size)],
			)

		for pkg in upgrade_names:
			name, old_version, new_version, size = pkg
			upgrade_info.append(
				[style(name, **BLUE),
				old_version,
				new_version,
				unit_str(size)],
			)

		pprint_names(['Package:', 'Version:', 'Size:'], delete_info, 'Removed:')
		pprint_names(['Package:', 'Version:', 'Size:'], install_info, 'Installed:')
		pprint_names(['Package:', 'Old Version:', 'New Version:', 'Size:'], upgrade_info, 'Upgraded:')

	def auto_remover(self):
		autoremove = []

		for pkg in self.cache:
			# We have to check both of these. Sometimes weird things happen
			if pkg.is_installed and pkg.is_auto_removable:
				pkg.mark_delete(purge=self.purge)
				autoremove.append(
					f"<Package: '{pkg.name}' Arch: '{pkg.installed.architecture}' Version: '{pkg.installed.version}'"
					)

		dprint(f"Pkgs marked by autoremove: {autoremove}")

	def check_essential(self, pkgs):
		for pkg in pkgs:
			if pkg.is_installed:
				# do not allow the removal of essential or required packages
				if pkg.installed.priority == 'required' and pkg.marked_delete:
					self.essential.append(pkg.name)
				# do not allow the removal of nala
				elif pkg.shortname in self.nala_depends and pkg.marked_delete:
					self.nala.append(pkg.name)

		if self.essential:
			pkg_error(self.essential, 'cannot be removed', banter='apt', terminate=True)
		if self.nala:
			pkg_error(self.nala, 'cannot be removed', banter='auto_preservation', terminate=True)

	def get_changes(self, upgrade=False, remove=False):
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
			self.check_essential(pkgs)
			self.print_update_summary(pkgs)
			if not self.assume_yes and not ask('Do you want to continue'):
				print("Abort.")
				return

			write_history(pkgs)
			write_log(pkgs)

			pkgs = [pkg for pkg in pkgs if not pkg.marked_delete and \
						not self.file_downloaded(pkg, hash_check = \
													  self.hash_check)]
			if self.metalink_out:
				with open(self.metalink_out, 'w', encoding='utf-8') as f:
					make_metalink(f, pkgs)
				return
			if not self.download(pkgs, num_concurrent=guess_concurrent(pkgs)):
				print("Some downloads failed. apt_pkg will take care of them.")

		if self.download_only:
			print("Download complete and in download only mode.")
		else:
			self.start_dpkg()

	def start_dpkg(self):
		# Lets get our environment variables set before we get down to business
		if self.noninteractive:
			environ["DEBIAN_FRONTEND"] = "noninteractive"
		if self.noninteractive_full:
			environ["DEBIAN_FRONTEND"] = "noninteractive"
			apt_pkg.config.set('Dpkg::Options::', '--force-confdef')
			apt_pkg.config.set('Dpkg::Options::', '--force-confold')
		if self.no_aptlist:
			environ["APT_LISTCHANGES_FRONTEND"] = "none"
		if self.confdef:
			apt_pkg.config.set('Dpkg::Options::', '--force-confdef')
		if self.confold:
			apt_pkg.config.set('Dpkg::Options::', '--force-confold')
		if self.confnew:
			apt_pkg.config.set('Dpkg::Options::', '--force-confnew')
		if self.confmiss:
			apt_pkg.config.set('Dpkg::Options::', '--force-confmiss')
		if self.confask:
			apt_pkg.config.set('Dpkg::Options::', '--force-confask')

		# If self.raw_dpkg is enabled likely they want to see the update too.
		# Turn off Rich scrolling if we don't have XTERM.
		if self.raw_dpkg or 'xterm' not in os.environ["TERM"]:
			self.verbose = True
			
		try:
			self.cache.commit(
				nalaProgress(self.verbose, self.debug),
				InstallProgress(self.verbose, self.debug)
			)
		except apt_pkg.Error as e:
			print(f'\r\n{style("Error:", **RED)} {e}')
			exit(1)

	def download(self, pkgs, num_concurrent=2):
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
				line = line.strip()
				if line != '':
					if self.debug:
						dprint(line)
					else:
						print(line)
		else:
			download_progress(pkgs, proc)

		proc.wait()
		link_success = True
		self.downloaded = True
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
	
	def file_downloaded(self, pkg, hash_check=False):
		candidate = pkg.candidate
		path = self.archive_dir / get_filename(candidate)
		if not path.exists() or path.stat().st_size != candidate.size:
			return False
		if not hash_check:
			return True
		hash_type, hash_value = get_hash(pkg.candidate)
		try:
			return check_hash(path, hash_type, hash_value)
		except OSError as e:
			if e.errno != errno.ENOENT:
				print("Failed to check hash", e)
			return False

	def print_update_summary(self, pkgs):

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
					pkg.installed.version,
					pkg.candidate.version,
					unit_str(pkg.candidate.size)]
				)

		delete = 'Remove'
		deleting = 'Removing:'
		if self.purge:
			delete = 'Purge'
			deleting = 'Purging:'

		pprint_names(['Package:', 'Version:', 'Size:'], delete_names, deleting)
		pprint_names(['Package:', 'Version:', 'Size:'], install_names, 'Installing:')
		pprint_names(['Package:', 'Old Version:', 'New Version:', 'Size:'], upgrade_names, 'Upgrading:')

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
		transaction_summary(delete_names, width, delete)
		
		if self.cache.required_download > 0:
			print(f'\nTotal download size: {unit_str(self.cache.required_download)}')
		else:
			# We need this extra line lol
			print()
		if self.cache.required_space < 0:
			print(f'Disk space to free: {unit_str(-self.cache.required_space)}')
		else:
			print(f'Disk space required: {unit_str(self.cache.required_space)}')
		if self.download_only:
			print("Nala will only download the packages")

def pkg_error(pkg_list: list, msg: str, banter: str = None, terminate: bool=False):
	"""
	banter is optional and can be one of::

		apt: "Maybe apt will let you"
		self_preservation: "Why do you think I would destroy myself?"
		auto_essential: "Whatever you did tried to auto mark essential packages"
		auto_preservation: "Whatever you did would have resulted in my own removal!"
	"""
	for pkg in pkg_list:
		print(
			style('Error:', **RED),
			style(pkg, **YELLOW),
			msg
		)

	if banter:
		if banter == 'apt':
			print(f"Maybe {style('apt', **RED)} will let you")

		elif banter == 'auto_essential':
			print("Whatever you did tried to auto mark essential packages")

		elif banter == 'auto_preservation':
			print("This would have resulted in my own removal!")

	if terminate:
		exit(1)

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
	mirrors = []
	for pkg in pkgs:
		version = pkg.candidate
		hashtype, hashvalue = get_hash(version)
		out.write(f'<file name="{get_filename(version)}">')
		out.write(f'<size>{version.size}</size>')
		if hashtype:
			out.write(f'<hash type="{hashtype}">{hashvalue}</hash>')
		for uri in version.uris:
			# To support mirrors.txt, and keep it fast we don't check if mirrors is already set
			if not mirrors and 'mirror://mirrors.ubuntu.com/mirrors.txt' in uri:
				try:
					mirrors = requests.get("http://mirrors.ubuntu.com/mirrors.txt").text.splitlines()
				except requests.ConnectionError:
					err = style("Error:", **RED)
					stderr.write(f'{err} unable to connect to http://mirrors.ubuntu.com/mirrors.txt\n')
					exit(1)
			# If we use mirrors we don't have to request it, we already have our list.
			if 'mirror://mirrors.ubuntu.com/mirrors.txt' in uri:
				for link in mirrors:
					link = uri.replace('mirror://mirrors.ubuntu.com/mirrors.txt', link)
					out.write(f'<url priority="1">{link}</url>')
					continue
			out.write(f'<url priority="1">{uri}</url>')
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
		justify = ['l', 'l', 'l', 'r'] if 'Upgrad' in title else ['l', 'l', 'r']
		print(columnar(names, headers, no_borders=True, justify=justify))

def transaction_summary(names, width, header: str):
	if names:
		print(
			header.ljust(7),
			f'{len(names)}'.rjust(width),
			'Packages'
			)

def unit_str(val, just = 7):
	if val > 1000**2:
		return f"{val/1000/1000 :.1f}".rjust(just)+" MB"
	elif val > 1000:
		return f"{round(val/1000) :.0f}".rjust(just)+" kB"
	else:
		return f'{val :.0f}'.rjust(just)+" B"

def write_history(pkgs):

	delete_names = []
	install_names = []
	upgrade_names = []

	# TODO marked_downgrade, marked_keep, marked_reinstall
	for pkg in pkgs:
		if pkg.marked_delete:
			delete_names.append(
				[pkg.name, pkg.installed.version, pkg.candidate.size]
			)

		elif pkg.marked_install:
			install_names.append(
				[pkg.name, pkg.candidate.version, pkg.candidate.size]
			)

		elif pkg.marked_upgrade:
			upgrade_names.append(
				[pkg.name, pkg.installed.version, pkg.candidate.version, pkg.candidate.size]
			)

	history = []
	if NALA_HISTORY.exists():
		history = NALA_HISTORY.read_text().splitlines()

	ID = len(history) + 1 if history else 1
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

def write_log(pkgs):
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
		iprint(f'Removed: {", ".join(delete_names)}')
	if install_names:
		iprint(f'Installed: {", ".join(install_names)}')
	if upgrade_names:
		iprint(f'Upgraded: {", ".join(upgrade_names)}')

	logger_newline()

def dep_format(package_dependecy):
	"""Takes a dependency object like pkg.candidate.dependencies"""
	for dep_list in package_dependecy:
		dep_print = ''
		for num, dep in enumerate(dep_list):
			open_paren = style('(', bold=True)
			close_paren = style(')', bold=True)
			name = style(dep.name, **GREEN)
			relation = style(dep.relation, bold=True)
			version = style(dep.version, **BLUE)
			pipe = style(' | ', bold=True)

			final = name+' '+open_paren+relation+' '+version+close_paren

			if num == 0:
				dep_print = '  '+final if dep.relation else '  '+name
			else:
				dep_print += pipe+final if dep.relation else pipe+name
		print(dep_print)

def download_progress(pkgs: list[Package], proc: Popen):

	# Add up the size of all our packages so we know the total
	total = sum(pkg.candidate.size for pkg in pkgs)
	task = pkg_download_progress.add_task(
		"[bold][blue]Downloading [green]Packages",
		total=total
		)

	with rich_live() as live:
		num = 0
		for line in iter(proc.stdout.readline, ''):

			try:
				deb_name = Path(pkgs[num].candidate.filename).name
			except IndexError:
				pass

			table = rich_grid()
			table.add_row(f'{style("Total Packages:", **GREEN)} {num}/{len(pkgs)}')
			table.add_row(f'{style("Current Package:", **GREEN)} {deb_name}')
			table.add_row(pkg_download_progress)
			live.update(table)

			if 'Download complete:' in line:
				pkg_download_progress.advance(task, advance=pkgs[num].candidate.size)
				num += 1
