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
import sys
from asyncio import CancelledError, run
from copy import deepcopy
from os import environ
from typing import NoReturn

import apt_pkg
from apt.cache import Cache, FetchFailedException, LockFailedException
from apt.debfile import DebPackage
from apt.package import Package

from nala.constants import (ARCHIVE_DIR, DPKG_LOG,
				ERROR_PREFIX, NALA_DIR, PARTIAL_DIR, ExitCode)
from nala.downloader import PkgDownloader
from nala.dpkg import InstallProgress, OpProgress, UpdateProgress, notice
from nala.history import write_history, write_log
from nala.install import (broken_error, check_broken,
				install_local, package_manager, split_local)
from nala.options import arguments
from nala.rich import Columns, Live, Table, Text, console, dpkg_progress
from nala.show import additional_notice, check_virtual, show
from nala.utils import (DelayedKeyboardInterrupt, ask,
				check_pkg, color, dprint, get_pkg_name, pkg_candidate,
				pkg_installed, print_packages, term, unit_str)


class Nala:
	"""Manage Nala operations."""

	def __init__(self,	no_update: bool = False) -> None:
		"""Manage Nala operations."""
		self.purge = False
		self.deleted: list[str] = []
		self.autoremoved: list[str] = []
		self.local_debs: list[DebPackage] = []
		# If raw_dpkg is enabled likely they want to see the update too.
		# Turn off Rich scrolling if we don't have XTERM.
		if arguments.raw_dpkg or not term.is_xterm():
			arguments.verbose = True
		self.cache = setup_cache(no_update)

	def upgrade(self, dist_upgrade: bool = False) -> None:
		"""Upgrade pkg[s]."""
		self.cache.upgrade(dist_upgrade=dist_upgrade)
		self.auto_remover()
		self.get_changes(upgrade=True)

	def install(self, pkg_names: list[str]) -> None:
		"""Install pkg[s]."""
		dprint(f"Install pkg_names: {pkg_names}")
		self.local_debs, pkg_names, not_exist = split_local(pkg_names, self.cache)
		local_names = install_local(self.local_debs) if self.local_debs else []

		pkg_names = glob_filter(pkg_names, self.cache.keys())

		broken, not_found = check_broken(pkg_names, self.cache)
		not_found.extend(not_exist)

		if not_found:
			pkg_error(not_found, 'not found', terminate=True)

		if not package_manager(pkg_names, self.cache):
			broken_error(broken)

		self.auto_remover()
		self.get_changes(local_names=local_names)

	def remove(self, pkg_names: list[str], purge: bool = False) -> None:
		"""Remove or Purge pkg[s]."""
		dprint(f"Remove pkg_names: {pkg_names}")
		not_found: list[str] = []
		self.purge = purge

		pkg_names = glob_filter(pkg_names, self.cache.keys())
		broken, not_found = check_broken(
			pkg_names, self.cache, remove=True, purge=purge
		)

		if not_found:
			pkg_error(not_found, 'not found')

		if not package_manager(
			pkg_names, self.cache,
			remove=True, deleted=self.deleted, purge=purge):

			broken_error(
				broken,
				tuple(pkg for pkg in self.cache if pkg.is_installed and pkg_installed(pkg).dependencies)
			)

		dprint(f"Marked delete: {self.deleted}")

		self.auto_remover()
		self.get_changes(remove=True)

	def show(self, pkg_names: list[str]) -> None:
		"""Show package information."""
		dprint(f"Show pkg_names: {pkg_names}")
		not_found: list[str] = []
		additional_records = 0
		for num, pkg_name in enumerate(pkg_names):
			if pkg_name in self.cache:
				pkg = self.cache[pkg_name]
				additional_records += show(num, pkg)
				continue

			if check_virtual(pkg_name, self.cache):
				continue
			not_found.append(f"{ERROR_PREFIX}{color(pkg_name, 'YELLOW')} not found")

		if additional_records and not arguments.all_versions:
			additional_notice(additional_records)

		if not_found:
			for error in not_found:
				print(error)
			sys.exit(1)

	def auto_remover(self) -> None:
		"""Handle auto removal of packages."""
		if not arguments.no_autoremove:
			for pkg in self.cache:
				# We have to check both of these. Sometimes weird things happen
				if pkg.is_installed and pkg.is_auto_removable and pkg.name not in self.deleted:
					pkg.mark_delete(purge=self.purge)
					self.autoremoved.append(pkg.name)

			dprint(f"Pkgs marked by autoremove: {self.autoremoved}")

	def get_changes(self,
		upgrade: bool = False, remove: bool = False, local_names: list[list[str]] | None = None) -> None:
		"""Get packages requiring changes and process them."""
		pkgs = sorted(self.cache.get_changes(), key=sort_pkg_name)
		if not NALA_DIR.exists():
			NALA_DIR.mkdir()

		check_work(pkgs, upgrade, remove)

		if pkgs:
			check_essential(pkgs)
			delete_names, install_names, upgrade_names, autoremove_names = self.sort_pkg_changes(pkgs)
			if local_names:
				install_names.extend(local_names)
			self.print_update_summary(delete_names, install_names, upgrade_names, autoremove_names)

			check_term_ask()

			# Calculate our total operations for the dpkg progress bar
			pkg_total = (
				len(delete_names)
				+ len(autoremove_names)
				+ len(install_names)*2
				+ len(upgrade_names)*2
			)
			pkgs = [
				# Don't download packages that already exist
				pkg for pkg in pkgs if not pkg.marked_delete and not check_pkg(ARCHIVE_DIR, pkg)
			]

		download(pkgs)

		write_history(delete_names+autoremove_names, install_names, upgrade_names)
		write_log(delete_names, install_names, upgrade_names, autoremove_names)
		self.start_dpkg(pkg_total)

	def start_dpkg(self, pkg_total: int) -> None:
		"""Set environment and start dpkg."""
		set_env()
		try:
			self.commit_pkgs(pkg_total)
		# Catch system error because if dpkg fails it'll throw this
		except (apt_pkg.Error, SystemError) as error:
			sys.exit(f'\r\n{ERROR_PREFIX + str(error)}')
		except FetchFailedException as error:
			for failed in str(error).splitlines():
				print(ERROR_PREFIX + failed)
			sys.exit(1)
		except KeyboardInterrupt:
			print("Exiting due to SIGINT")
			sys.exit(ExitCode.SIGINT)
		finally:
			term.restore_mode()
			# If dpkg quits for any reason we lose the cursor
			term.write(term.SHOW_CURSOR+term.CLEAR_LINE)
			if notice:
				print('\n'+color('Notices:', 'YELLOW'))
				for notice_msg in notice:
					print(notice_msg)
		print(color("Finished Successfully", 'GREEN'))

	def commit_pkgs(self, pkg_total: int) -> None:
		"""Commit the package changes to the cache."""
		if self.local_debs:
			# Add one because we start a new instance of InstallProgress
			pkg_total += 1

		task = dpkg_progress.add_task('', total=pkg_total + 1)
		with Live(auto_refresh=False) as live:
			with open(DPKG_LOG, 'w', encoding="utf-8") as dpkg_log:
				if arguments.raw_dpkg:
					live.stop()
				self.cache.commit(
					UpdateProgress(live, install=True),
					InstallProgress(dpkg_log, live, task)
				)
				for deb in self.local_debs:
					deb.install(InstallProgress(dpkg_log, live, task))

	def sort_pkg_changes(self, pkgs: list[Package]
		) -> tuple[list[list[str]], list[list[str]], list[list[str]], list[list[str]]]:
		"""Sort a list of packages and splits them based on the action to take."""
		delete_names: list[list[str]] = []
		install_names: list[list[str]] = []
		upgrade_names: list[list[str]] = []
		autoremove_names: list[list[str]] = []

		for pkg in pkgs:
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
				continue

			candidate = pkg_candidate(pkg)
			if pkg.marked_install:
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

def setup_cache(no_update: bool) -> Cache:
	"""Update the cache if necessary, and then return the Cache."""
	try:
		if not no_update:
			with DelayedKeyboardInterrupt():
				with Live(auto_refresh=False) as live:
					Cache().update(UpdateProgress(live))
	except (LockFailedException, FetchFailedException) as err:
		apt_error(err)
	except KeyboardInterrupt:
		print('Exiting due to SIGINT')
		sys.exit(ExitCode.SIGINT)
	finally:
		term.restore_mode()
		term.write(term.SHOW_CURSOR+term.CLEAR_LINE)
	return Cache(OpProgress())

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

def check_essential(pkgs: list[Package]) -> None:
	"""Check removal of essential packages."""
	if arguments.remove_essential:
		return
	essential: list[Text] = []
	for pkg in pkgs:
		if pkg.is_installed:
			# do not allow the removal of essential or required packages
			if pkg_installed(pkg).priority == 'required' and pkg.marked_delete:
				essential.append(
					Text.from_ansi(color(pkg.name, 'RED'))
				)
			# do not allow the removal of nala
			elif pkg.shortname in 'nala' and pkg.marked_delete:
				essential.append(
					Text.from_ansi(color('nala', 'RED'))
				)

	if essential:
		essential_error(essential)

def essential_error(pkg_list: list[Text]) -> NoReturn:
	"""Print error message for essential packages and exit."""
	the_following = color('The Following Packages are')
	essential_package = f"You have attempted to remove {color('essential packages', 'RED')}"
	if len(pkg_list) == 1:
		the_following = color('The Following Package is')
		essential_package = f"You have attempted to remove an {color('essential package', 'RED')}"
	print('='*term.columns)
	print(the_following, color('Essential!', 'RED'))
	print('='*term.columns)
	console.print(Columns(pkg_list, padding=(0,2), equal=True))

	print('='*term.columns)
	switch = color('--remove-essential', 'YELLOW')
	print(ERROR_PREFIX+essential_package)
	print(ERROR_PREFIX+f"Please use {switch} if you are sure you want too.")
	sys.exit(1)

def pkg_error(pkg_list: list[str], msg: str = '', terminate: bool = False) -> None:
	"""Print error for package in list."""
	for pkg in pkg_list:
		print(ERROR_PREFIX+color(pkg, 'YELLOW'), msg)

	if terminate:
		sys.exit(1)

def process_downloads(pkgs: list[Package]) -> bool:
	"""Process the downloaded packages."""
	link_success = True
	for pkg in pkgs:
		filename = get_pkg_name(pkg_candidate(pkg))
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

def set_env() -> None:
	"""Set environment."""
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

def download(pkgs: list[Package]) -> None:
	"""Run downloads and check for failures.

	Does not return if in Download Only mode.
	"""
	downloader = PkgDownloader(pkgs)
	try:
		run(downloader.start_download())
	except CancelledError as error:
		if downloader.exit:
			sys.exit(downloader.exit)
		raise error

	if arguments.download_only:
		if downloader.failed:
			for pkg in downloader.failed:
				print(ERROR_PREFIX+f"{pkg} Failed to download")
			sys.exit(ERROR_PREFIX+'Some downloads failed and in download only mode.')

		print("Download complete and in download only mode.")
		sys.exit(0)

	if downloader.failed:
		print("Some downloads failed. Falling back to apt_pkg.")

def glob_filter(pkg_names: list[str], cache_keys: list[str]) -> list[str]:
	"""Filter provided packages and glob *.

	Returns a new list of packages matching the glob.

	If there is nothing to glob it returns the original list.
	"""
	if '*' not in str(pkg_names):
		return pkg_names

	new_packages: list[str] = []
	glob_failed = False
	for pkg_name in pkg_names:
		if '*' in pkg_name:
			dprint(f'Globbing: {pkg_name}')
			glob = fnmatch.filter(cache_keys, pkg_name)
			if not glob:
				glob_failed = True
				print(ERROR_PREFIX+f"unable to find any packages by globbing {color(pkg_name, 'YELLOW')}")
				continue
			new_packages.extend(
				fnmatch.filter(cache_keys, pkg_name)
			)
		else:
			new_packages.append(pkg_name)

	dprint(f'List after globbing: {new_packages}')
	if glob_failed:
		sys.exit(1)
	return new_packages

def apt_error(apt_err: FetchFailedException | LockFailedException) -> NoReturn:
	"""Take an error message from python-apt and formats it."""
	msg = str(apt_err)
	if not msg:
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
	if not term.is_su():
		sys.exit('Are you root?')
	sys.exit(1)
