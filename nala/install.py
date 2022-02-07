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

from nala.constants import ERROR_PREFIX
from nala.show import print_dep
from nala.utils import color, dprint, pkg_installed, term


def install_pkg(pkg: Package) -> None:
	"""Mark package for installation or upgrade."""
	if not pkg.installed:
		pkg.mark_install()
		dprint(f"Marked Install: {pkg.name}")
	elif pkg.is_upgradable:
		pkg.mark_upgrade()
		dprint(f"Marked upgrade: {pkg.name}")

def remove_pkg(pkg: Package, deleted: list[str], purge: bool = False) -> None:
	"""Mark package for removal."""
	if pkg.installed:
		pkg.mark_delete(purge=purge)
		dprint(f"Marked Remove: {pkg.name}")
		deleted.append(pkg.name)

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

def install_local(local_debs: list[DebPackage]) -> list[list[str]]:
	"""Mark the depends for local debs to be installed.

	Dependencies that are marked will be marked auto installed.

	Returns local_names to be printed in the transaction summary.
	"""
	local_names: list[list[str]] = []
	failed = False
	for pkg in local_debs:
		if not check_local_version(pkg):
			continue
		if not pkg.check():
			failed = True
			local_missing_dep(pkg)
			continue

		local_names.append(
			[pkg.pkgname, pkg._sections["Version"], pkg._sections["Installed-Size"]]
		)
	if failed:
		sys.exit(1)
	return local_names

def check_local_version(pkg: DebPackage) -> bool:
	"""Check if the version installed is better than the .deb."""
	if pkg_compare := pkg.compare_to_version_in_cache() == pkg.VERSION_SAME:
		print(
			f"Package {color(pkg.pkgname, 'GREEN')}",
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
	pkg_names: list[str], cache: Cache) -> tuple[list[DebPackage], list[str], list[str]]:
	"""Split pkg_names into either Local debs, regular install or they don't exist."""
	local_debs: list[DebPackage] = []
	cache_debs: list[str] = []
	not_exist: list[str] = []
	for name in pkg_names:
		if '.deb' in name:
			if not Path(name).exists():
				not_exist.append(name)
				continue
			local_debs.append(
				DebPackage(name, cache)
			)
			continue
		cache_debs.append(name)
	return local_debs, cache_debs, not_exist

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
						raise error
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
	sys.exit(ERROR_PREFIX + 'You have held broken packages')

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
