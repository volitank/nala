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
"""Functions for the Nala Install command."""
from __future__ import annotations

import os
import sys
from pathlib import Path
from typing import Iterable, cast

import apt_pkg
from apt.cache import Cache, FetchFailedException, LockFailedException
from apt.debfile import DebPackage
from apt.package import Package
from apt_pkg import DepCache, Error as AptError

from nala.constants import (ARCHIVE_DIR, DPKG_LOG, ERROR_PREFIX, NALA_DIR,
				NALA_TERM_LOG, NEED_RESTART, REBOOT_PKGS, REBOOT_REQUIRED, ExitCode)
from nala.downloader import download
from nala.dpkg import InstallProgress, OpProgress, UpdateProgress, notice
from nala.error import apt_error, essential_error
from nala.history import write_history, write_log
from nala.options import arguments
from nala.rich import Live, Text, dpkg_progress, from_ansi
from nala.utils import (DelayedKeyboardInterrupt, NalaPackage,
				PackageHandler, ask, check_pkg, color, dprint, get_date,
				pkg_candidate, pkg_installed, print_update_summary, term, vprint)


def install_pkg(pkg: Package) -> None:
	"""Mark package for installation or upgrade."""
	if not pkg.installed:
		pkg.mark_install(auto_fix=arguments.no_fix_broken)
		dprint(f"Marked Install: {pkg.name}")
	elif pkg.is_upgradable:
		pkg.mark_upgrade()
		dprint(f"Marked upgrade: {pkg.name}")

def remove_pkg(pkg: Package, deleted: list[str], purge: bool = False) -> None:
	"""Mark package for removal."""
	if pkg.installed:
		pkg.mark_delete(auto_fix=arguments.no_fix_broken, purge=purge)
		dprint(f"Marked Remove: {pkg.name}")
		deleted.append(pkg.name)

def auto_remover(cache: Cache, nala_pkgs: PackageHandler, purge: bool = False) -> None:
	"""Handle auto removal of packages."""
	if not arguments.no_autoremove:
		for pkg in cache:
			# We have to check both of these. Sometimes weird things happen
			if pkg.is_installed and pkg.is_auto_removable and pkg.name not in nala_pkgs.deleted:
				pkg.mark_delete(auto_fix=arguments.no_fix_broken, purge=purge)
				nala_pkgs.autoremoved.append(pkg.name)

		dprint(f"Pkgs marked by autoremove: {nala_pkgs.autoremoved}")

def commit_pkgs(cache: Cache, nala_pkgs: PackageHandler) -> None:
	"""Commit the package changes to the cache."""
	task = dpkg_progress.add_task('', total=nala_pkgs.dpkg_progress_total + 1)
	with Live(auto_refresh=False) as live:
		with open(DPKG_LOG, 'w', encoding="utf-8") as dpkg_log:
			with open(NALA_TERM_LOG, 'a', encoding="utf-8") as term_log:
				term_log.write(f"Log Started: [{get_date()}]\n")
				if arguments.raw_dpkg:
					live.stop()
				cache.commit(
					UpdateProgress(live, install=True),
					InstallProgress(dpkg_log, term_log, live, task)
				)
				for deb in nala_pkgs.local_debs:
					deb.install(InstallProgress(dpkg_log, term_log, live, task))
				term_log.write(f"Log Ended: [{get_date()}]\n\n")

def get_changes(cache: Cache, nala_pkgs: PackageHandler,
	upgrade: bool = False, remove: bool = False) -> None:
	"""Get packages requiring changes and process them."""
	pkgs = sorted(cache.get_changes(), key=sort_pkg_name)
	if not NALA_DIR.exists():
		NALA_DIR.mkdir()

	check_work(pkgs, nala_pkgs.local_debs, upgrade, remove)

	if pkgs or nala_pkgs.local_debs:
		check_essential(pkgs)
		sort_pkg_changes(pkgs, nala_pkgs)
		print_update_summary(nala_pkgs, cache)

		check_term_ask()

		pkgs = [
			# Don't download packages that already exist
			pkg for pkg in pkgs if not pkg.marked_delete and not check_pkg(ARCHIVE_DIR, pkg)
		]

	download(pkgs)

	write_history(nala_pkgs)
	write_log(nala_pkgs)
	start_dpkg(cache, nala_pkgs)

def start_dpkg(cache: Cache, nala_pkgs: PackageHandler) -> None:
	"""Start dpkg."""
	try:
		commit_pkgs(cache, nala_pkgs)
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

def local_missing_dep(pkg: DebPackage) -> None:
	"""Print missing depends for .deb that can't be satisfied."""
	cache = Cache(memonly=True)
	print(color(cast(str, pkg.filename), 'YELLOW'), "can not be installed")
	print(color('Missing Depends:', 'YELLOW'))
	for or_group in pkg.depends:
		for depend in or_group:
			name, ver, operand = depend
			if name not in cache:
				print(f"  {color (name, 'GREEN')} {color(operand)} {color(ver, 'BLUE')}")

def install_local(nala_pkgs: PackageHandler) -> None:
	"""Mark the depends for local debs to be installed.

	Dependencies that are marked will be marked auto installed.

	Returns local_names to be printed in the transaction summary.
	"""
	failed = False
	for pkg in nala_pkgs.local_debs[:]:
		check_local_version(pkg, nala_pkgs)
		if not pkg.check(allow_downgrade=True):
			failed = True
			local_missing_dep(pkg)
			nala_pkgs.local_debs.remove(pkg)
			continue
		if pkg.pkgname not in [pkg.name for pkg in nala_pkgs.upgrade_pkgs]:
			nala_pkgs.local_pkgs.append(
				NalaPackage(pkg.pkgname, pkg._sections["Version"], int(pkg._sections["Installed-Size"]))
			)
	if failed:
		sys.exit(1)

def check_local_version(pkg: DebPackage, nala_pkgs: PackageHandler) -> None:
	"""Check if the version installed is better than the .deb."""
	if pkg_compare := pkg.compare_to_version_in_cache():
		cache_pkg = pkg._cache[pkg.pkgname]

		if pkg_compare == pkg.VERSION_SAME and cache_pkg.is_installed:
			deb_ver = color(pkg._sections['Version'], 'BLUE')
			print(f"{color(pkg.pkgname, 'GREEN')} {deb_ver} will be re-installed")
			return

		# Eventually we'll make a Downgrades and Re-Install Column
		if pkg_compare == pkg.VERSION_OUTDATED:
			install_ver = color(pkg_installed(cache_pkg).version, 'BLUE')
			print(f"Package {color(pkg.pkgname, 'GREEN')} {install_ver} will be downgraded")
			return

		if pkg_compare == pkg.VERSION_NEWER and cache_pkg.is_installed:
			nala_pkgs.upgrade_pkgs.append(
				NalaPackage(
					pkg.pkgname, pkg._sections['Version'],
					int(pkg._sections["Installed-Size"]), pkg_installed(cache_pkg).version
				)
			)

def split_local(
	pkg_names: list[str], cache: Cache, local_debs: list[DebPackage]) -> list[str]:
	"""Split pkg_names into either Local debs, regular install or they don't exist."""
	not_exist: list[str] = []
	for name in pkg_names[:]:
		if '.deb' in name:
			if not Path(name).exists():
				not_exist.append(name)
				pkg_names.remove(name)
				continue
			local_debs.append(
				DebPackage(name, cache)
			)
			pkg_names.remove(name)
			continue
	return not_exist

def package_manager(pkg_names: list[str], cache: Cache,
	deleted: list[str] | None = None, remove: bool = False, purge: bool = False) -> bool:
	"""Manage installation or removal of packages."""
	with cache.actiongroup(): # type: ignore[attr-defined]
		for pkg_name in pkg_names:
			if pkg_name in cache:
				pkg = cache[pkg_name]
				try:
					if remove:
						assert isinstance(deleted, list)
						remove_pkg(pkg, deleted, purge=purge)
						continue
					install_pkg(pkg)
				except AptError as error:
					if ('broken packages' not in str(error)
					and 'held packages' not in str(error)):
						raise error from error
					return False
	return True

def get_extra_pkgs(extra_type: str, pkg_names: list[str],
	cache: Cache, npkg_list: list[NalaPackage | list[NalaPackage]]) -> None:
	"""Get Recommended or Suggested Packages."""
	for pkg_name in pkg_names:
		pkg = cache[pkg_name]
		if pkg.candidate and (recommends := pkg.candidate.get_dependencies(extra_type)):
			for dep in recommends:
				if len(dep) == 1:
					ver = dep.target_versions[0]
					npkg_list.append(
						NalaPackage(ver.package.name, ver.version, ver.size)
					)
					continue
				npkg_list.append(
					[NalaPackage(
						base_dep.target_versions[0].package.name,
						base_dep.target_versions[0].version,
						base_dep.target_versions[0].size
					) for base_dep in dep if base_dep.target_versions]
				)

def check_broken(pkg_names: list[str], cache: Cache,
	remove: bool = False, purge: bool = False) -> tuple[list[Package], list[str]]:
	"""Check if packages will be broken."""
	broken_count = 0
	not_found: list[str] = []
	broken: list[Package] = []
	depcache = cache._depcache

	with cache.actiongroup(): # type: ignore[attr-defined]
		for pkg_name in pkg_names:
			if pkg_name not in cache:
				not_found.append(pkg_name)
				continue

			pkg = cache[pkg_name]
			mark_pkg(pkg, depcache, remove=remove, purge=purge)
			if depcache.broken_count > broken_count:
				broken.append(pkg)
				broken_count += 1
	return broken, not_found

def mark_pkg(pkg: Package, depcache: DepCache,
	remove: bool = False, purge: bool = False) -> bool:
	"""Mark Packages in depcache for broken checks."""
	if remove:
		if not pkg.installed:
			print(ERROR_PREFIX+color(pkg.name, 'YELLOW'), 'not installed')
			return False
		depcache.mark_delete(pkg._pkg, purge)
		return True

	if pkg.installed and not pkg.is_upgradable:
		print(
			f"Package {color(pkg.name, 'GREEN')}",
			'is already at the latest version',
			color(pkg.installed.version, 'BLUE')
			)
		return False
	depcache.mark_install(pkg._pkg, False, True)
	return True

def installed_missing_dep(pkg: Package) -> None:
	"""Print missing deps for broken package."""
	if pkg.installed:
		for depends in pkg_installed(pkg).dependencies:
			for dep in depends:
				if not dep.target_versions:
					ver_msg = f"{color(dep.name, 'YELLOW')} {color(dep.relation_deb)} {color(dep.version, 'BLUE')}"
					print(f"{color(pkg.name, 'GREEN')} is missing {ver_msg}")
		if pkg.marked_delete:
			print(F"{color(pkg.name, 'GREEN')} will be {color('removed', 'RED')}")

def installed_found_deps(pkg: Package, cache: Cache) -> None:
	"""Print depends that will be installed to fix the package."""
	vprint(f"{color(pkg.name, 'GREEN')} is fixable")
	for depends in pkg_installed(pkg).dependencies:
		for dep in depends:
			if cache[dep.name].marked_install:
				vprint(f"  {color(dep.name, 'GREEN')} will be {color('installed', 'GREEN')}")

def sort_pkg_changes(pkgs: list[Package], nala_pkgs: PackageHandler) -> None:
	"""Sort a list of packages and splits them based on the action to take."""
	for pkg in pkgs:
		if pkg.marked_delete:
			installed = pkg_installed(pkg)
			if pkg.name not in nala_pkgs.autoremoved:
				nala_pkgs.delete_pkgs.append(
					NalaPackage(pkg.name, installed.version, installed.size),
				)
			else:
				nala_pkgs.autoremove_pkgs.append(
					NalaPackage(pkg.name, installed.version, installed.size)
				)
			continue

		candidate = pkg_candidate(pkg)
		if pkg.marked_install:
			nala_pkgs.install_pkgs.append(
				NalaPackage(pkg.name, candidate.version, candidate.size)
			)

		elif pkg.marked_upgrade:
			installed = pkg_installed(pkg)
			nala_pkgs.upgrade_pkgs.append(
				NalaPackage(
					pkg.name, candidate.version,
					candidate.size, old_version=installed.version
				)
			)

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
		print('\n'+color('Notices:', 'YELLOW'))
		for notice_msg in notices:
			print(notice_msg)

def setup_cache() -> Cache:
	"""Update the cache if necessary, and then return the Cache."""
	if arguments.no_install_recommends:
		apt_pkg.config.set('APT::Install-Recommends', '0')
	if arguments.install_suggests:
		apt_pkg.config.set('APT::Install-Suggests', '1')
	set_env()
	try:
		if not check_update():
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

def check_update() -> bool:
	"""Check if we should update the cache or not."""
	no_update_list = ('remove', 'show', 'search', 'history', 'install', 'purge')
	no_update = cast(bool, arguments.no_update)
	if arguments.command in no_update_list:
		no_update = True
	if not arguments.command and arguments.fix_broken:
		no_update = True
	if arguments.update:
		no_update = False
	return no_update

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

def set_env() -> None:
	"""Set environment."""
	if arguments.non_interactive:
		os.environ["DEBIAN_FRONTEND"] = "noninteractive"
	if arguments.non_interactive_full:
		os.environ["DEBIAN_FRONTEND"] = "noninteractive"
		apt_pkg.config.set('Dpkg::Options::', '--force-confdef')
		apt_pkg.config.set('Dpkg::Options::', '--force-confold')
	if arguments.no_aptlist:
		os.environ["APT_LISTCHANGES_FRONTEND"] = "none"
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
