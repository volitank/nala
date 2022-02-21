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
import re
import sys
from asyncio import CancelledError, run
from os import environ
from typing import Iterable, NoReturn, Pattern

import apt_pkg
from apt.cache import Cache, FetchFailedException, LockFailedException
from apt.debfile import DebPackage
from apt.package import Package, Version

from nala.constants import (ARCHIVE_DIR, ERROR_PREFIX, NALA_DIR,
				NEED_RESTART, PARTIAL_DIR, REBOOT_PKGS, REBOOT_REQUIRED, ExitCode)
from nala.downloader import PkgDownloader
from nala.dpkg import OpProgress, UpdateProgress, notice
from nala.history import write_history, write_log
from nala.install import (auto_remover, broken_error,
				check_broken, commit_pkgs, get_extra_pkgs, install_local,
				installed_found_deps, installed_missing_dep, package_manager,
				print_update_summary, sort_pkg_changes, split_local)
from nala.options import arguments
from nala.rich import (Columns, Live, Text,
				Tree, escape, from_ansi, search_progress)
from nala.show import additional_notice, check_virtual, show
from nala.utils import (DelayedKeyboardInterrupt, ask,
				check_pkg, color, dprint, get_pkg_name, get_version,
				nala_pkgs, pkg_candidate, pkg_installed, term)


class Nala:
	"""Manage Nala operations."""

	def __init__(self,	no_update: bool = False) -> None:
		"""Manage Nala operations."""
		# If raw_dpkg is enabled likely they want to see the update too.
		# Turn off Rich scrolling if we don't have XTERM.
		if arguments.raw_dpkg or not term.is_xterm():
			arguments.verbose = True
		self.cache = setup_cache(no_update)

	def upgrade(self, dist_upgrade: bool = False) -> None:
		"""Upgrade pkg[s]."""
		is_upgrade = [pkg for pkg in self.cache if pkg.is_upgradable]
		self.cache.upgrade(dist_upgrade=dist_upgrade)

		if kept_back := [pkg for pkg in is_upgrade if not pkg.is_upgradable]:
			for pkg in kept_back:
				print(f"{color(pkg.name, 'YELLOW')} was kept back")
			check_term_ask()

		auto_remover(self.cache, nala_pkgs)
		self.get_changes(upgrade=True)

	def install(self, pkg_names: list[str]) -> None:
		"""Install pkg[s]."""
		dprint(f"Install pkg_names: {pkg_names}")
		not_exist = split_local(pkg_names, self.cache, nala_pkgs.local_debs)
		install_local(nala_pkgs)

		pkg_names = glob_filter(pkg_names, self.cache.keys())
		broken, not_found = check_broken(pkg_names, self.cache)
		not_found.extend(not_exist)

		if not_found:
			pkg_error(not_found, 'not found', terminate=True)

		if arguments.no_install_recommends:
			get_extra_pkgs('Recommends', pkg_names, self.cache, nala_pkgs.recommend_pkgs)
		if not arguments.install_suggests:
			get_extra_pkgs('Suggests', pkg_names, self.cache, nala_pkgs.suggest_pkgs)

		if not package_manager(pkg_names, self.cache):
			broken_error(broken)

		auto_remover(self.cache, nala_pkgs)
		self.get_changes()

	def remove(self, pkg_names: list[str], purge: bool = False) -> None:
		"""Remove or Purge pkg[s]."""
		dprint(f"Remove pkg_names: {pkg_names}")
		not_found: list[str] = []

		pkg_names = glob_filter(pkg_names, self.cache.keys())
		broken, not_found = check_broken(
			pkg_names, self.cache, remove=True, purge=purge
		)

		if not_found:
			pkg_error(not_found, 'not found')

		if not package_manager(
			pkg_names, self.cache,
			remove=True, deleted=nala_pkgs.deleted, purge=purge):

			broken_error(
				broken,
				tuple(pkg for pkg in self.cache if pkg.is_installed and pkg_installed(pkg).dependencies)
			)

		dprint(f"Marked delete: {nala_pkgs.deleted}")

		auto_remover(self.cache, nala_pkgs, purge)
		self.get_changes(remove=True)

	def fix_broken(self) -> None:
		"""Attempt to fix broken packages, if any."""
		broken: list[Package] = []
		fixable: list[Package] = []
		fixer = apt_pkg.ProblemResolver(self.cache._depcache)
		for pkg in self.cache:
			if pkg.is_now_broken:
				try:
					pkg.mark_install()
					fixable.append(pkg)
				except apt_pkg.Error as error:
					if 'broken packages' not in str(error):
						raise error from error
					broken.append(pkg)
					self.cache.clear()
					fixer.clear(pkg._pkg)
					fixer.resolve(True)

		for pkg in broken:
			installed_missing_dep(pkg)
		for pkg in fixable:
			installed_found_deps(pkg, self.cache)

		auto_remover(self.cache, nala_pkgs)
		self.get_changes()

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

	def search(self, search_term: str) -> None:
		"""Search command entry point."""
		found: list[tuple[Package, Version]] = []
		if search_term == '*':
			search_term = '.*'
		search_pattern = re.compile(search_term, re.IGNORECASE)
		with search_progress as progress:
			task = progress.add_task('Searching...', total=len(self.cache))
			arches = apt_pkg.get_architectures()
			for pkg in self.cache:
				if arguments.installed and not pkg.installed:
					progress.advance(task)
					continue
				if pkg.architecture() in arches:
					search_name(pkg, search_pattern, found)
				progress.advance(task)
		if not found:
			print(f"{ERROR_PREFIX}{color(search_term, 'YELLOW')} was not found.")
		for item in found:
			print_search(*item)

	def get_changes(self,
		upgrade: bool = False, remove: bool = False) -> None:
		"""Get packages requiring changes and process them."""
		pkgs = sorted(self.cache.get_changes(), key=sort_pkg_name)
		if not NALA_DIR.exists():
			NALA_DIR.mkdir()

		check_work(pkgs, nala_pkgs.local_debs, upgrade, remove)

		if pkgs or nala_pkgs.local_debs:
			check_essential(pkgs)
			sort_pkg_changes(pkgs, nala_pkgs)
			print_update_summary(nala_pkgs, self.cache)

			check_term_ask()

			pkgs = [
				# Don't download packages that already exist
				pkg for pkg in pkgs if not pkg.marked_delete and not check_pkg(ARCHIVE_DIR, pkg)
			]

		download(pkgs)

		write_history(nala_pkgs)
		write_log(nala_pkgs)
		self.start_dpkg()

	def start_dpkg(self) -> None:
		"""Start dpkg."""
		try:
			commit_pkgs(self.cache, nala_pkgs)
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
			print_notices(notice)
			if need_reboot():
				print(f"{color('Notice:', 'YELLOW')} A reboot is required.")
		print(color("Finished Successfully", 'GREEN'))

def need_reboot() -> bool:
	"""Check if the system needs a reboot and notify the user."""
	if REBOOT_REQUIRED.exists():
		if REBOOT_PKGS.exists():
			print(f"{color('Notice:', 'YELLOW')} The following packages require a reboot.")
			for pkg in REBOOT_PKGS.read_text(encoding='utf-8').splitlines():
				print(f"  {color(pkg, 'GREEN')}")
			return False
		return True
	if NEED_RESTART.exists():
		return True
	return False

def print_notices(notices: Iterable[str]) -> None:
	"""Print notices from dpkg."""
	if notices:
		if REBOOT_REQUIRED.exists() or NEED_RESTART.exists():
			if not (notices := [msg for msg in notices if 'reboot' not in msg]):
				return
		print('\n'+color('Notices:', 'YELLOW'))
		for notice_msg in notices:
			print(notice_msg)

def setup_cache(no_update: bool) -> Cache:
	"""Update the cache if necessary, and then return the Cache."""
	if arguments.no_install_recommends:
		apt_pkg.config.set('APT::Install-Recommends', '0')
	if arguments.install_suggests:
		apt_pkg.config.set('APT::Install-Suggests', '1')
	set_env()
	try:
		if not no_update:
			with DelayedKeyboardInterrupt():
				with Live(auto_refresh=False) as live:
					Cache().update(UpdateProgress(live))
	except (LockFailedException, FetchFailedException, apt_pkg.Error) as err:
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
		sys.exit(
			f'{ERROR_PREFIX}It can be dangerous to continue without a terminal. Use `--assume-yes`'
		)

	if not arguments.assume_yes and not ask('Do you want to continue'):
		print("Abort.")
		sys.exit(0)

def check_work(pkgs: list[Package], local_debs: list[DebPackage],
	upgrade: bool, remove: bool) -> None:
	"""Check if there is any work for nala to do.

	Returns None if there is work, exit's successful if not.
	"""
	if upgrade and not pkgs:
		print(color("All packages are up to date."))
		sys.exit(0)
	elif not remove and not pkgs and not local_debs:
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
			if pkg.essential and pkg.marked_delete:
				essential.append(
					from_ansi(color(pkg.name, 'RED'))
				)
			# do not allow the removal of nala
			elif pkg.shortname in 'nala' and pkg.marked_delete:
				essential.append(
					from_ansi(color('nala', 'RED'))
				)

	if essential:
		essential_error(essential)

def search_name(pkg: Package,
	search_pattern: Pattern[str], found: list[tuple[Package, Version]]) -> None:
	"""Search the package name and description."""
	version = get_version(pkg)
	searches = [pkg.name]
	if not arguments.names:
		searches.extend([version.raw_description, version.source_name])
	for string in searches:
		if re.findall(search_pattern, string):
			found.append((pkg, version))
			break

def print_search(pkg: Package, version: Version) -> None:
	"""Print the search results to the terminal."""
	first_line = f"{color(pkg.name, 'GREEN')} {color(version.version, 'BLUE')}"
	tree = get_search_origin(first_line, version)
	if pkg.is_installed:
		if not pkg.is_upgradable:
			tree.add('is installed')
		else:
			tree.add(f"is upgradable from {color(pkg_installed(pkg).version, 'BLUE')}")

	if arguments.full and version._translated_records:
		tree.add(version._translated_records.long_desc)
	else:
		tree.add(f"{version.raw_description.splitlines()[0]}")
	term.console.print(tree)
	print()

def get_search_origin(first_line: str, version: Version) -> Tree:
	"""Return the origin of the package to print."""
	if (origin := version.origins[0]).component == 'now':
		return Tree(f"{first_line} {escape('[local]')}")
	return Tree(
		f"{first_line} {escape(f'[{origin.label}/{origin.codename} {origin.component}]')}"
	)


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
	term.console.print(Columns(pkg_list, padding=(0,2), equal=True))

	print('='*term.columns)
	switch = color('--remove-essential', 'YELLOW')
	print(ERROR_PREFIX+essential_package)
	print(f'{ERROR_PREFIX}Please use {switch} if you are sure you want too.')
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
				print(f'{ERROR_PREFIX}Failed to move archive file {err}')
			link_success = False
	return link_success

def set_env() -> None:
	"""Set environment."""
	if arguments.non_interactive:
		environ["DEBIAN_FRONTEND"] = "noninteractive"
	if arguments.non_interactive_full:
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
		raise error from error

	if arguments.download_only:
		if downloader.failed:
			for pkg in downloader.failed:
				print(f'{ERROR_PREFIX}{pkg} Failed to download')
			sys.exit(f'{ERROR_PREFIX}Some downloads failed and in download only mode.')

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
				print(
					f'{ERROR_PREFIX}unable to find any packages by globbing {color(pkg_name, "YELLOW")}'
				)
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

def apt_error(apt_err: FetchFailedException | LockFailedException | apt_pkg.Error) -> NoReturn:
	"""Take an error message from python-apt and formats it."""
	msg = str(apt_err)
	if not msg:
		# Sometimes python apt gives us literally nothing to work with.
		# Probably an issue with sources.list. Needs further testing.
		sys.exit(
			f"{ERROR_PREFIX}python-apt gave us '{repr(apt_err)}'\nThis isn't a proper error as it's empty"
			)
	if ',' in msg:
		err_list = set(msg.split(','))
		for err in err_list:
			if 'E:' in err:
				err = err.replace('E:', '')
				print(ERROR_PREFIX+err.strip())
				continue
			if 'W:' in err:
				err = err.replace('W:', '')
				print(color('Warning: ', 'YELLOW')+err.strip())
				continue
		sys.exit(1)
	print(ERROR_PREFIX+msg)
	if not term.is_su():
		sys.exit('Are you root?')
	sys.exit(1)
