#                 __
#    ____ _____  |  | _____
#   /    \\__  \ |  | \__  \
#  |   |  \/ __ \|  |__/ __ \_
#  |___|  (____  /____(____  /
#       \/     \/          \/
#
# Copyright (C) 2010 Tatsuhiro Tsujikawa
# Copyright (C) 2021, 2022 Blake Lee
#
# This file is part of nala
# nala is based upon apt-metalink https://github.com/tatsuhiro-t/apt-metalink
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
"""Main module for Nala which facilitates apt."""
from __future__ import annotations

import errno
import fnmatch
import hashlib
import re
import sys
from concurrent.futures import ThreadPoolExecutor
from copy import deepcopy
from os import environ
from pathlib import Path
from random import shuffle
from shutil import copyfileobj
from typing import NoReturn, Pattern

import apt_pkg
import requests  # type: ignore[import]
from apt.cache import Cache, FetchFailedException, LockFailedException
from apt.package import Package, Version

from nala.constants import ARCHIVE_DIR, ERROR_PREFIX, NALA_DIR, PARTIAL_DIR
from nala.dpkg import InstallProgress, UpdateProgress
from nala.history import write_history, write_log
from nala.logger import dprint
from nala.options import arguments
from nala.rich import Live, Table, pkg_download_progress
from nala.show import check_virtual, show_main
from nala.utils import (ask, color, pkg_candidate, pkg_installed,
				print_packages, term, unit_str, verbose_print)


class Nala:
	"""Manage Nala operations."""

	def __init__(self,	no_update: bool = False) -> None:
		"""Manage Nala operations."""
		self.purge = False
		self.deleted: list[list[str]] = []
		self.autoremoved: list[list[str]] = []
		# If raw_dpkg is enabled likely they want to see the update too.
		# Turn off Rich scrolling if we don't have XTERM.
		if arguments.raw_dpkg or not term.is_xterm():
			arguments.verbose = True
		# We want to update the cache before we initialize it
		if not no_update:
			print('Updating package list...')
			try:
				Cache().update(UpdateProgress())
			except (LockFailedException, FetchFailedException) as err:
				apt_error(err)
		try:
			self.cache = Cache(UpdateProgress())
		except (LockFailedException, FetchFailedException) as err:
			apt_error(err)

	def upgrade(self, dist_upgrade: bool = False) -> None:
		"""Upgrade pkg[s]."""
		self.cache.upgrade(dist_upgrade=dist_upgrade)
		self.auto_remover()
		self.get_changes(upgrade=True)

	def glob_filter(self, pkg_names: list[str]) -> list[str]:
		"""Filter provided packages and glob *."""
		packages = self.cache.keys()
		new_packages: list[str] = []

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

	def install(self, pkg_names: list[str]) -> None:
		"""Install pkg[s]."""
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
					print(f"Package {color(pkg.name, 'GREEN')}",
					'is already at the latest version',
					color(pkg.installed.version, 'BLUE'))

		if not_found:
			pkg_error(not_found, 'not found', terminate=True)

		self.auto_remover()
		self.get_changes()

	def remove(self, pkg_names: list[str], purge: bool = False) -> None:
		"""Remove or Purge pkg[s]."""
		dprint(f"Remove pkg_names: {pkg_names}")
		not_found: list[str] = []
		not_installed: list[str] = []
		self.purge = purge

		# We only want to glob if we detect an *
		if '*' in str(pkg_names):
			pkg_names = self.glob_filter(pkg_names)

		for pkg_name in pkg_names:
			if check_found(self.cache, pkg_name, not_found, not_installed):

				pkg = self.cache[pkg_name]
				pkg.mark_delete(purge=self.purge)
				# Add name to deleted for autoremove checking.
				self.deleted.append(pkg.name)
		dprint(f"Marked delete: {self.deleted}")

		if not_installed:
			pkg_error(not_installed, 'not installed')
		if not_found:
			pkg_error(not_found, 'not found')

		self.auto_remover()
		self.get_changes(remove=True)

	def show(self, pkg_names: list[str]) -> None:
		"""Show package information."""
		dprint(f"Show pkg_names: {pkg_names}")
		for num, pkg_name in enumerate(pkg_names):
			if pkg_name in self.cache:
				if num > 0:
					print()
				show_main(self.cache[pkg_name])
			else:
				check_virtual(pkg_name, self.cache)
				sys.exit(f"{ERROR_PREFIX}{color(pkg_name, 'YELLOW')} not found")

	def auto_remover(self) -> None:
		"""Handle auto removal of packages."""
		if not arguments.no_autoremove:
			for pkg in self.cache:
				# We have to check both of these. Sometimes weird things happen
				if pkg.is_installed and pkg.is_auto_removable and pkg.name not in self.deleted:
					pkg.mark_delete(purge=self.purge)
					self.autoremoved.append(pkg.name)

			dprint(f"Pkgs marked by autoremove: {self.autoremoved}")

	def get_changes(self, upgrade: bool = False, remove: bool = False) -> None:
		"""Get packages requiring changes and process them."""
		pkgs = sorted(self.cache.get_changes(), key=sort_pkg_name)
		if not NALA_DIR.exists():
			NALA_DIR.mkdir()

		check_work(pkgs, upgrade, remove)

		if pkgs:
			check_essential(pkgs)
			delete_names, install_names, upgrade_names, autoremove_names = self.sort_pkg_changes(pkgs)
			self.print_update_summary(delete_names, install_names, upgrade_names, autoremove_names)

			check_term_ask()

			pkgs = [
				# Don't download packages that already exist
				pkg for pkg in pkgs if not pkg.marked_delete and not check_pkg(ARCHIVE_DIR, pkg)
			]

			if not PkgDownloader(pkgs).download():
				print("Some downloads failed. apt_pkg will take care of them.")

		if arguments.download_only:
			print("Download complete and in download only mode.")
		else:
			write_history(delete_names, install_names, upgrade_names)
			write_log(delete_names, install_names, upgrade_names, autoremove_names)
			self.start_dpkg()

	def start_dpkg(self) -> None:
		"""Set environment and start dpkg."""
		# Lets get our environment variables set before we get down to business
		if arguments.noninteractive:
			environ["DEBIAN_FRONTEND"] = "noninteractive"
		if arguments.noninteractive_full:
			environ["DEBIAN_FRONTEND"] = "noninteractive"
			apt_pkg.config.set('Dpkg::Options::', '--force-confdef')
			apt_pkg.config.set('Dpkg::Options::', '--force-confold')
		if arguments.no_aptlist:
			environ["APT_LISTCHANGES_FRONTEND"] = "none"
		if arguments.confdef:
			apt_pkg.config.set('Dpkg::Options::', '--force-confdef')
		if arguments.confold:
			apt_pkg.config.set('Dpkg::Options::', '--force-confold')
		if arguments.confnew:
			apt_pkg.config.set('Dpkg::Options::', '--force-confnew')
		if arguments.confmiss:
			apt_pkg.config.set('Dpkg::Options::', '--force-confmiss')
		if arguments.confask:
			apt_pkg.config.set('Dpkg::Options::', '--force-confask')

		try:
			self.cache.commit(UpdateProgress(), InstallProgress())
		except apt_pkg.Error as err:
			sys.exit(f'\r\n{ERROR_PREFIX+str(err)}')

	def sort_pkg_changes(self, pkgs: list[Package]
		) -> tuple[list[list[str]], list[list[str]], list[list[str]], list[list[str]]]:
		"""Sort a list of packages and splits them based on the action to take."""
		delete_names: list[list[str]] = []
		install_names: list[list[str]] = []
		upgrade_names: list[list[str]] = []
		autoremove_names: list[list[str]] = []

		for pkg in pkgs:
			candidate = pkg_candidate(pkg)
			if pkg.marked_delete:
				installed = pkg_installed(pkg)
				if pkg.name not in self.autoremoved:
					delete_names.append(
						[pkg.name, installed.version, str(installed.size)]
					)
				else:
					autoremove_names.append(
						[pkg.name, installed.version, str(installed.size)]
					)

			elif pkg.marked_install:
				install_names.append(
					[pkg.name, candidate.version, str(candidate.size)]
				)

			elif pkg.marked_upgrade:
				installed = pkg_installed(pkg)
				upgrade_names.append(
					[pkg.name, installed.version, candidate.version, str(candidate.size)]
				)
		return delete_names, install_names, upgrade_names, autoremove_names

	def print_update_summary(self,
			delete_names: list[list[str]], install_names: list[list[str]],
			upgrade_names: list[list[str]], autoremove_names:list[list[str]]) -> None:
		"""Print our transaction summary."""
		delete = ('Purge', 'Purging:') if self.purge else ('Remove', 'Removing:')

		print_packages(
			['Package:', 'Version:', 'Size:'],
			deepcopy(delete_names), delete[1], 'bold red'
		)

		print_packages(
			['Package:', 'Version:', 'Size:'],
			deepcopy(autoremove_names), 'Auto-Removing:', 'bold red'
		)

		print_packages(
			['Package:', 'Version:', 'Size:'],
			deepcopy(install_names), 'Installing:', 'bold green'
		)

		print_packages(
			['Package:', 'Old Version:', 'New Version:', 'Size:'],
			deepcopy(upgrade_names), 'Upgrading:', 'bold blue'
		)

		transaction_summary(delete[0],
			len(delete_names), len(install_names),
			len(upgrade_names), len(autoremove_names))
		self.transaction_footer()

	def transaction_footer(self) -> None:
		"""Print transaction footer."""
		print()
		if self.cache.required_download > 0:
			print(f'Total download size: {unit_str(self.cache.required_download)}')
		if self.cache.required_space < 0:
			print(f'Disk space to free: {unit_str(-int(self.cache.required_space))}')
		else:
			print(f'Disk space required: {unit_str(self.cache.required_space)}')
		if arguments.download_only:
			print("Nala will only download the packages")

def sort_pkg_name(pkg: Package) -> str:
	"""Sort by package name.

	This is to be used as sorted(key=sort_pkg_name)
	"""
	return str(pkg.name)

def check_term_ask() -> None:
	"""Check terminal and ask user if they want to continue."""
	# If we're piped or something the user should specify --assume-yes
	# As They are aware it can be dangerous to continue
	if not term.is_term() and not arguments.assume_yes:
		sys.exit(ERROR_PREFIX+"It can be dangerous to continue without a terminal. Use `--assume-yes`")

	if not arguments.assume_yes and not ask('Do you want to continue'):
		print("Abort.")
		sys.exit(0)

def check_work(pkgs: list[Package], upgrade: bool, remove: bool) -> None:
	"""Check if there is any work for nala to do.

	Returns None if there is work, exit's successful if not.
	"""
	if upgrade and not pkgs:
		print(color("All packages are up to date."))
		sys.exit(0)
	elif not remove and not pkgs:
		print(color("Nothing for Nala to do."))
		sys.exit(0)
	elif remove and not pkgs:
		print(color("Nothing for Nala to remove."))
		sys.exit(0)

def check_found(cache: Cache, pkg_name: str,
	not_found: list[str], not_installed: list[str]) -> bool:
	"""Check if package is in the cache or installed.

	Return True if the package is found.
	"""
	if pkg_name not in cache:
		not_found.append(pkg_name)
		return False

	pkg = cache[pkg_name]
	if not pkg.installed:
		not_installed.append(pkg_name)
		return False
	return True

def check_essential(pkgs: list[Package]) -> None:
	"""Check removal of essential packages."""
	essential: list[str] = []
	nala_check: bool = False
	banter: str = 'apt'
	for pkg in pkgs:
		if pkg.is_installed:
			# do not allow the removal of essential or required packages
			if pkg_installed(pkg).priority == 'required' and pkg.marked_delete:
				essential.append(pkg.name)
			# do not allow the removal of nala
			elif pkg.shortname in 'nala' and pkg.marked_delete:
				nala_check = True
				banter = 'auto_preservation'
				essential.append('nala')

	if essential or nala_check:
		pkg_error(essential, 'cannot be removed', banter=banter, terminate=True)

def pkg_error(pkg_list: list[str], msg: str, banter: str = '', terminate: bool = False) -> None:
	"""Print error for package in list.

	banter is optional and can be one of::

		apt: "Maybe apt will let you"
		auto_essential: "Whatever you did tried to auto mark essential packages"
		auto_preservation: "Whatever you did would have resulted in my own removal!"
	"""
	for pkg in pkg_list:
		print(ERROR_PREFIX+color(pkg, 'YELLOW'), msg)
	if banter:
		if banter == 'apt':
			print(f"Maybe {color('apt', 'RED')} will let you")

		elif banter == 'auto_essential':
			print("Whatever you did tried to auto mark essential packages")

		elif banter == 'auto_preservation':
			print("This would have resulted in my own removal!")
	if terminate:
		sys.exit(1)

def check_pkg(directory: Path, candidate: Package | Version) -> bool:
	"""Check if file exists, is correct, and run check hash."""
	if isinstance(candidate, Package):
		candidate = pkg_candidate(candidate)
	path = directory / Path(candidate.filename).name
	if not path.exists() or path.stat().st_size != candidate.size:
		return False
	hash_type, hash_value = get_hash(candidate)
	try:
		return check_hash(path, hash_type, hash_value)
	except OSError as err:
		if err.errno != errno.ENOENT:
			print("Failed to check hash", err)
		return False

def check_hash(path: Path, hash_type: str, hash_value: str) -> bool:
	"""Check hash value."""
	hash_fun = hashlib.new(hash_type)
	with path.open('rb') as file:
		while True:
			data = file.read(4096)
			if not data:
				break
			hash_fun.update(data)
	local_hash = hash_fun.hexdigest()
	debugger = (
		str(path),
		f"Candidate Hash: {hash_type} {hash_value}",
		f"Local Hash: {local_hash}"
	)
	dprint(debugger)
	return local_hash == hash_value

def get_hash(version: Version) -> tuple[str, str]:
	"""Get the correct hash value."""
	if version.sha256:
		return ("sha256", version.sha256)
	if version.sha1:
		return ("sha1", version.sha1)
	if version.md5:
		return ("md5", version.md5)
	sys.exit(ERROR_PREFIX+f"{Path(version.filename).name} can't be checked for integrity.")

def process_downloads(pkgs: list[Package]) -> bool:
	"""Process the downloaded packages."""
	link_success = True
	for pkg in pkgs:
		filename = Path(pkg_candidate(pkg).filename).name
		destination = ARCHIVE_DIR / filename
		source = PARTIAL_DIR / filename
		try:
			dprint(f"Moving {source} to {destination}")
			source.rename(destination)
		except OSError as err:
			if err.errno != errno.ENOENT:
				print(ERROR_PREFIX+f"Failed to move archive file {err}")
			link_success = False
	return link_success

def filter_uris(candidate: Version, mirrors: list[str], pattern: Pattern[str]) -> list[str]:
	"""Filter uris into usable urls."""
	urls: list[str] = []
	for uri in candidate.uris:
		# Regex to check if we're using mirror.txt
		regex = pattern.search(uri)
		if regex:
			domain = regex.group(1)
			if not mirrors:
				try:
					mirrors = requests.get(f"http://{domain}/mirrors.txt").text.splitlines()
				except requests.ConnectionError:
					sys.exit(ERROR_PREFIX+f'unable to connect to http://{domain}/mirrors.txt')
			urls.extend([link+candidate.filename for link in mirrors])
			continue
		urls.append(uri)
	return urls

def guess_concurrent(pkgs: list[Package]) -> int:
	"""Determine how many concurrent downloads to do."""
	max_uris = 2
	for pkg in pkgs:
		candidate = pkg_candidate(pkg)
		max_uris = max(len(candidate.uris)*2, max_uris)
	return max_uris

def transaction_summary(delete_header: str,
	delete_total: int, install_total: int,
	upgrade_total: int, autoremove_total: int) -> None:
	"""Print a small transaction summary."""
	print('='*term.columns)
	print('Summary')
	print('='*term.columns)
	table = Table.grid('', padding=(0,2))
	table.add_column(justify='right')
	table.add_column()

	if install_total:
		table.add_row('Install', str(install_total), 'Packages')
	if upgrade_total:
		table.add_row('Upgrade', str(upgrade_total), 'Packages')
	if delete_total:
		table.add_row(delete_header, str(delete_total), 'Packages')
	if autoremove_total:
		table.add_row('Auto-Remove', str(autoremove_total), 'Packages')
	term.console.print(table)

def apt_error(apt_err: FetchFailedException | LockFailedException) -> NoReturn:
	"""Take an error message from python-apt and formats it."""
	msg = str(apt_err)
	if msg == '':
		# Sometimes python apt gives us literally nothing to work with.
		# Probably an issue with sources.list. Needs further testing.
		sys.exit(
			ERROR_PREFIX+f"python-apt gave us '{repr(apt_err)}'\n"
			"This isn't a proper error as it's empty"
			)
	if '.,' in msg:
		err_list = set(msg.split(','))
		for err in err_list:
			err = err.replace('E:', '')
			print(ERROR_PREFIX+err.strip())
		sys.exit(1)
	print(ERROR_PREFIX+msg)
	sys.exit('Are you root?')

class PkgDownloader:
	"""Manage Package Downloads."""

	def __init__(self, pkgs: list[Package]) -> None:
		"""Manage Package Downloads."""
		self.pkgs = pkgs
		self.total_size: int = sum(pkg_candidate(pkg).size for pkg in self.pkgs)
		self.total_pkgs: int = len(self.pkgs)
		self.count: int = 0
		self.live: Live
		self.task = pkg_download_progress.add_task(
			"[bold][blue]Downloading [green]Packages",
			total=self.total_size
		)

		http_proxy = apt_pkg.config.find('Acquire::http::Proxy')
		https_proxy = apt_pkg.config.find('Acquire::https::Proxy', http_proxy)
		ftp_proxy = apt_pkg.config.find('Acquire::ftp::Proxy')

		self.proxy: dict[str, str] = {
			'http' : http_proxy,
			'https' : https_proxy,
			'ftp' : ftp_proxy
		}

	def download(self) -> bool:
		"""Download pkgs."""
		with Live() as self.live:
			# We don't want to use more than 16 threads.
			threads = min(guess_concurrent(self.pkgs), 16)
			with ThreadPoolExecutor(max_workers=threads) as pool:
				mirrors: list[str] = []
				pattern = re.compile('mirror://([A-Za-z_0-9.-]+).*')
				for pkg in self.pkgs:
					urls: list[str] = []
					candidate = pkg_candidate(pkg)
					urls = filter_uris(candidate, mirrors, pattern)
					# Randomize the urls to minimize load on a single mirror.
					shuffle(urls)
					self._start_thread(urls, pool, candidate)
		return process_downloads(self.pkgs)

	@staticmethod
	def check_integrity(candidate: Version, filename: str) -> None:
		"""Check package integrity. Raise IntegrityError if issue."""
		dprint(f'Checking integrity: {filename}')
		if not check_pkg(PARTIAL_DIR, candidate):
			dprint(f'Integrity check failed: {filename}')
			raise IntegrityError("This is only to catch")
		dprint(f'Integrity check successful: {filename}')

	def _download_pkg(self, candidate: Version, url: str) -> None:
		"""Download package and update progress."""
		dest = PARTIAL_DIR / Path(candidate.filename).name
		verbose_print(f"{color('Starting Download:', 'GREEN')} {url}")
		with requests.get(url, stream=True) as download:
			download.raise_for_status()
			with dest.open('wb') as file:
				copyfileobj(download.raw, file)
		self.check_integrity(candidate, dest.name)

		verbose_print(f"{color('Finished Download:', 'GREEN')} {dest.name}")
		self._update_progress(dest.name, candidate.size)

	def _start_thread(self,
		urls: list[str], pool: ThreadPoolExecutor, candidate: Version) -> None:
		"""Start download thread."""
		for _ in urls:
			try:
				url = urls.pop(0)
				pool.submit(self._download_pkg, candidate, url)
				break
			except (requests.ConnectionError, requests.HTTPError):
				print("There was a problem connecting. Trying the next URL")
			except IntegrityError:
				continue
			except IndexError:
				sys.exit(ERROR_PREFIX+f'There are no more URLs to try for {Path(candidate.filename).name}..')

	def _gen_table(self, pkg_name: str) -> Table:
		"""Generate Rich Table."""
		table = Table.grid()
		table.add_row(f"{color('Total Packages:', 'GREEN')} {self.count}/{self.total_pkgs}")
		table.add_row(f"{color('Last Completed:', 'GREEN')} {pkg_name}")
		table.add_row(pkg_download_progress.get_renderable())
		return table

	def _update_progress(self, pkg_name: str, size: int) -> None:
		"""Update download progress."""
		pkg_download_progress.advance(self.task, advance=size)
		self.count += 1
		self.live.update(
			self._gen_table(pkg_name)
		)

class IntegrityError(Exception):
	"""Exception for integrity checking."""
