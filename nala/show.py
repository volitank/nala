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
"""Functions related to the `show` command."""
from __future__ import annotations

import sys
from pathlib import Path
from random import shuffle

from apt.cache import Cache
from apt.package import BaseDependency, Dependency, Package, Version

from nala.constants import PACSTALL_METADATA
from nala.utils import color, pkg_candidate, unit_str


def show_main(pkg: Package) -> None:
	"""Start show functions."""
	candidate = pkg_candidate(pkg)
	for pkg_info in show_format(pkg, candidate):
		if pkg_info:
			print(pkg_info)

	show_related(candidate)
	if candidate.homepage:
		print(color('Homepage:'), candidate.homepage)
	if candidate.size:
		print(color('Download-Size:'), unit_str(candidate.size, 1))
	print(format_sources(candidate, pkg))
	if candidate._translated_records:
		print(color('Description:'), candidate._translated_records.long_desc)

def check_virtual(pkg_name: str, cache: Cache) -> None:
	"""Check if the package is virtual."""
	virtual = [
		color(pkg.name, 'GREEN') for pkg in cache
		if pkg_name in pkg_candidate(pkg).provides
	]
	if virtual:
		print(
			color(pkg_name, 'YELLOW'),
			"is a virtual package satisfied by the following:\n"
			f"{', '.join(virtual)}"
		)
		sys.exit(0)

def show_related(candidate: Version) -> None:
	"""Show relational packages."""
	if candidate.provides:
		print_dep(
			color('Provides:'),
			[color(name, 'GREEN') for name in candidate.provides],
		)

	if candidate.enhances:
		print_dep(
			color('Enhances:'),
			[color(pkg[0].name, 'GREEN') for pkg in candidate.enhances],
		)

	if candidate.dependencies:
		depends, pre_depends = split_deps(candidate.dependencies)
		if pre_depends:
			print_dep(color('Pre-Depends:'), pre_depends)
		if depends:
			print_dep(color('Depends:'), depends)

	if candidate.recommends:
		print_dep(color('Recommends:'), candidate.recommends)

	if candidate.suggests:
		print_dep(color('Suggests:'), candidate.suggests)

	additional_related(candidate)

def additional_related(candidate: Version) -> None:
	"""Show breaks, conflicts, replaces."""
	breaks = candidate.get_dependencies('Breaks')
	conflicts = candidate.get_dependencies('Conflicts')
	replaces = candidate.get_dependencies('Replaces')

	if replaces:
		print_dep(
			color('Replaces:'),
			[color(pkg[0].name, 'GREEN') for pkg in replaces],
		)
	if conflicts:
		print_dep(color('Conflicts:'), conflicts)
	if breaks:
		print_dep(color('Breaks:'), breaks)

def show_format(pkg: Package, candidate: Version) -> tuple[str, ...]:
	"""Format main section for show command."""
	installed = 'yes' if pkg.is_installed else 'no'
	essential = 'yes' if pkg.essential else 'no'
	original_maintainer, bugs, origin, installed_size = filter_empty(candidate)

	return (
		f"{color('Package:')} {color(pkg.name, 'GREEN')}",
		f"{color('Version:')} {color(candidate.version, 'BLUE')}",
		f"{color('Architecture:')} {candidate.architecture}",
		f"{color('Installed:')} {installed}",
		f"{color('Priority:')} {candidate.priority}",
		f"{color('Essential:')} {essential}",
		f"{color('Section:')} {candidate.section}",
		f"{color('Source:')} {candidate.source_name}",
		origin,
		f"{color('Maintainer:')} {candidate.record.get('Maintainer')}",
		original_maintainer,
		bugs,
		installed_size,
	)

def filter_empty(candidate: Version) -> tuple[str, str, str, str]:
	"""Filter empty information blocks."""
	original_maintainer = candidate.record.get('Original-Maintainer')
	bugs = candidate.record.get('Bugs')
	origin = candidate.origins[0].origin
	installed_size = candidate.installed_size

	return (
		f"{color('Original-Maintainer:')} {original_maintainer}" if original_maintainer else '',
		f"{color('Bugs:')} {bugs}" if bugs else '',
		f"{color('Origin:')} {origin}" if origin else '',
		f"{color('Installed-Size:')} {unit_str(installed_size, 1)}" if installed_size else ''
	)

def print_dep(prefix: str,
	package_dependecy: list[Dependency] | list[str]) -> None:
	"""Print dependencies for show."""
	if isinstance(package_dependecy[0], str):
		package_dependecy.sort()
		package_dependecy = [str(dep) for dep in package_dependecy]
		print(prefix, ", ".join(package_dependecy))
		return

	join_list = []
	same_line = True
	if len(package_dependecy) > 4:
		same_line = False
		print(prefix)

	for dep_list in package_dependecy:
		dep_print = ''
		for num, dep in enumerate(dep_list):
			assert isinstance(dep, BaseDependency)
			if num == 0:
				dep_print = format_dep(dep, num)
			else:
				dep_print += format_dep(dep, num)

		if same_line:
			join_list.append(dep_print.strip())
			continue
		print(dep_print)

	if same_line:
		print(prefix,", ".join(join_list))

def format_dep(dep: BaseDependency, iteration: int) -> str:
	"""Format dependencies for show."""
	open_paren = color('(')
	close_paren = color(')')
	name = color(dep.name, 'GREEN')
	if dep.rawtype in ('Breaks', 'Conflicts'):
		name = color(dep.name, 'RED')
	relation = color(dep.relation)
	version = color(dep.version, 'BLUE')
	indent = color(' | ') if iteration > 0 else '  '

	final = name+' '+open_paren+relation+' '+version+close_paren

	return indent+final if dep.relation else indent+name

def format_sources(candidate: Version, pkg: Package) -> str:
	"""Show apt sources."""
	origin = candidate.origins[0]
	if origin.archive == 'now':
		return f'{color("APT-Sources:")} {get_local_source(pkg.shortname)}'

	return (
		f"{color('APT-Sources:')} {source_url(candidate.uris)} {origin.archive}/"
		f"{origin.component} {candidate.architecture} Packages"
	)

def source_url(uris: list[str]) -> str:
	"""Return the source url."""
	for mirror in uris:
		if 'mirror://' in mirror:
			index = mirror.index('/pool')
			return mirror[:index]

	shuffle(uris)
	index = uris[0].index('/pool')
	return uris[0][:index]

def get_local_source(pkg_name: str) -> str:
	"""Determine the local source and return it."""
	postfixes = ('', '-deb', '-git', '-bin', '-app')
	for postfix in postfixes:
		metadata = PACSTALL_METADATA / (pkg_name + postfix)
		if metadata.exists():
			return parse_pacstall(metadata)
	return 'local install'

def parse_pacstall(pacdata: Path) -> str:
	"""Parse pacstall metadata file."""
	remote = '_remoterepo='
	# _remoterepo="https://github.com/pacstall/pacstall-programs"
	for line in pacdata.read_text().splitlines():
		if line.startswith(remote):
			index = line.index('=') + 1
			return color(line[index:].strip('"'), 'BLUE')
	return color('https://github.com/pacstall/pacstall-programs', 'BLUE')

def split_deps(depend_list: list[Dependency]) -> tuple[list[Dependency], list[Dependency]]:
	"""Split dependencies into pre-depends and depends."""
	depends: list[Dependency] = []
	pre_depends: list[Dependency] = []
	for depend in depend_list:
		if depend[0].pre_depend:
			pre_depends.append(depend)
			continue
		depends.append(depend)
	return depends, pre_depends
