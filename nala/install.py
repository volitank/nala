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

import sys
from pathlib import Path
from typing import NoReturn, cast

from apt import Cache, Package, Version
from apt.debfile import DebPackage
from apt_pkg import DepCache, Error as AptError

from nala.constants import DPKG_LOG, ERROR_PREFIX
from nala.dpkg import InstallProgress, UpdateProgress
from nala.options import arguments
from nala.rich import Live, Table, dpkg_progress
from nala.show import print_dep
from nala.utils import (NalaPackage, PackageHandler, color, dprint,
				pkg_candidate, pkg_installed, print_packages, term, unit_str, vprint)


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
			if arguments.raw_dpkg:
				live.stop()
			cache.commit(
				UpdateProgress(live, install=True),
				InstallProgress(dpkg_log, live, task)
			)
			for deb in nala_pkgs.local_debs:
				deb.install(InstallProgress(dpkg_log, live, task))

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
		if not check_local_version(pkg):
			nala_pkgs.local_debs.remove(pkg)
			continue
		if not pkg.check():
			failed = True
			local_missing_dep(pkg)
			nala_pkgs.local_debs.remove(pkg)
			continue

		nala_pkgs.local_pkgs.append(
			NalaPackage(pkg.pkgname, pkg._sections["Version"], int(pkg._sections["Installed-Size"]))
		)
	if failed:
		sys.exit(1)

def check_local_version(pkg: DebPackage) -> bool:
	"""Check if the version installed is better than the .deb."""
	if pkg_compare := pkg.compare_to_version_in_cache() == pkg.VERSION_SAME:
		print(
			f"{color(pkg.pkgname, 'GREEN')}",
			'is already at the latest version',
			color(pkg._sections["Version"], 'BLUE')
		)
		return False

	if pkg_compare == pkg.VERSION_NEWER:
		deb_ver = (
			color('(') + color(pkg._sections['Version'], 'BLUE') + color(')')
		)
		install_ver = (
			color('(') + pkg_installed(pkg._cache[pkg.pkgname]).version + color(')')
		)
		print(
			f"Package {color(pkg.pkgname, 'GREEN')}",
			f"is older {deb_ver} than the version installed {install_ver}"
		)
		return False
	return True

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

def get_installed_dep_names(installed_pkgs: tuple[Package, ...]) -> tuple[str, ...]:
	"""Iterate installed pkgs and return all of their deps in a list.

	This is so we can reduce iterations when checking reverse depends.
	"""
	total_deps = []
	for pkg in installed_pkgs:
		for deps in pkg_installed(pkg).dependencies:
			for dep in deps:
				if dep.name not in total_deps:
					total_deps.append(dep.name)
	return tuple(total_deps)

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

def broken_error(broken: list[Package],
	installed_pkgs: bool | tuple[Package, ...] = False) -> NoReturn:
	"""Handle printing of errors due to broken packages."""
	if isinstance(installed_pkgs, tuple):
		total_deps = get_installed_dep_names(installed_pkgs)

	for pkg in broken:
		if pkg.candidate is None:
			for version in pkg.versions:
				print_broken(pkg.name, version)
				if installed_pkgs and pkg.name in total_deps:
					print_rdeps(
						pkg.name,
						cast(tuple[Package], installed_pkgs)
					)
			continue

		print_broken(pkg.name, pkg.candidate)
		if installed_pkgs and pkg.name in total_deps:
			print_rdeps(
				pkg.name,
				cast(tuple[Package], installed_pkgs)
			)

	print(f"\n{color('Notice:', 'YELLOW')} The information above may be able to help")
	sys.exit(f'{ERROR_PREFIX}You have held broken packages')

def print_broken(pkg_name: str, candidate: Version) -> None:
	"""Print broken packages information."""
	print('='*term.columns)
	version = color('(') + color(candidate.version, 'BLUE') + color(')')
	print(f"{color('Package:', 'YELLOW')} {color(pkg_name, 'GREEN')} {version}")
	if conflicts := candidate.get_dependencies('Conflicts'):
		print_dep(color('Conflicts:', 'YELLOW'), conflicts)
	if breaks := candidate.get_dependencies('Breaks'):
		print_dep(color('Breaks:', 'YELLOW'), breaks)
	if candidate.dependencies:
		print_dep(color('Depends:', 'YELLOW'), candidate.dependencies)

def print_rdeps(name: str, installed_pkgs: tuple[Package]) -> None:
	"""Print the installed reverse depends of a package."""
	print(color('Installed Packages Depend On This:', 'YELLOW'))
	for pkg in installed_pkgs:
		for dep in pkg_installed(pkg).dependencies:
			if name in dep.rawstr:
				print(' ', color(pkg.name, 'GREEN'))
				break

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
					pkg.name, installed.version,
					candidate.size, old_version=candidate.version
				)
			)

def print_update_summary(nala_pkgs: PackageHandler, cache: Cache, history: bool = False) -> None:
	"""Print our transaction summary."""
	delete, deleting = (
		('Purge', 'Purging:')
		if arguments.command == 'purge'
		else ('Remove', 'Removing:')
	)
	print_packages(
		['Package:', 'Version:', 'Size:'],
		nala_pkgs.delete_pkgs, deleting, 'bold red'
	)

	print_packages(
		['Package:', 'Version:', 'Size:'],
		nala_pkgs.autoremove_pkgs, 'Auto-Removing:', 'bold red'
	)

	print_packages(
		['Package:', 'Version:', 'Size:'],
		nala_pkgs.extended_install, 'Installing:', 'bold green'
	)

	print_packages(
		['Package:', 'Old Version:', 'New Version:', 'Size:'],
		nala_pkgs.upgrade_pkgs, 'Upgrading:', 'bold blue'
	)

	transaction_summary(delete, nala_pkgs, history)
	if not history:
		transaction_footer(cache)

def transaction_summary(
	delete_header: str, nala_pkgs: PackageHandler, history: bool = False) -> None:
	"""Print a small transaction summary."""
	print('='*term.columns)
	print('Summary')
	print('='*term.columns)
	table = Table.grid('', padding=(0,2))
	table.add_column(justify='right')
	table.add_column()

	if nala_pkgs.install_total:
		table.add_row(
			'Install' if not history else 'Installed',
			str(nala_pkgs.install_total), 'Packages')

	if nala_pkgs.upgrade_total:
		table.add_row(
			'Upgrade' if not history else 'Upgraded',
			str(nala_pkgs.upgrade_total),'Packages')

	if nala_pkgs.delete_total:
		table.add_row(delete_header, str(nala_pkgs.delete_total), 'Packages')

	if nala_pkgs.autoremove_total:
		table.add_row('Auto-Remove', str(nala_pkgs.autoremove_total), 'Packages')
	term.console.print(table)

def transaction_footer(cache: Cache) -> None:
	"""Print transaction footer."""
	print()
	if cache.required_download > 0:
		print(f'Total download size: {unit_str(cache.required_download)}')
	if cache.required_space < 0:
		print(f'Disk space to free: {unit_str(-int(cache.required_space))}')
	else:
		print(f'Disk space required: {unit_str(cache.required_space)}')
	if arguments.download_only:
		print("Nala will only download the packages")
