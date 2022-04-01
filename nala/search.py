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
"""Functions for the Nala Search command."""
from __future__ import annotations

import re
from typing import Iterable, Pattern, cast

from apt.package import Package, Version

from nala import COLOR_CODES, _, color
from nala.options import arguments
from nala.rich import ascii_replace, is_utf8
from nala.utils import get_version, pkg_installed

TOP_LINE = "├──" if is_utf8 else "+--"
BOT_LINE = "└──" if is_utf8 else "`--"


def search_name(
	pkg: Package,
	version: Version,
	search_pattern: Pattern[str],
	found: list[tuple[Package, Version]],
) -> None:
	"""Search the package name and description."""
	searches = [pkg.name]
	if not arguments.names:
		searches.extend([version.raw_description, version.source_name])
	for string in searches:
		if re.findall(search_pattern, string):
			found.append((pkg, version))
			break


def iter_search(found: Iterable[tuple[Package, Version | tuple[Version, ...]]]) -> bool:
	"""Iterate the search results."""
	pkg_list_check: list[Package] = []
	for item in found:
		pkg, version = item
		if isinstance(version, tuple):
			for ver in version:
				print_search(pkg, ver, pkg_list_check)
		else:
			print_search(pkg, version, pkg_list_check)

	if not pkg_list_check:
		return False
	return True


def print_search(pkg: Package, version: Version, pkg_list_check: list[Package]) -> None:
	"""Print the search results to the terminal."""
	pkg_list_check.append(pkg)
	print(
		ascii_replace(
			set_search_description(
				set_search_installed(
					set_search_origin(
						f"{color(pkg.name, 'GREEN')} {color(version.version, 'BLUE')}",
						cast(Version, get_version(pkg, cand_first=True)),
					),
					TOP_LINE,
					pkg,
					version,
				),
				BOT_LINE,
				version,
			)
		),
		end="\n\n",
	)


def set_search_origin(line: str, version: Version) -> str:
	"""Return the provided string with the origin information."""
	if origin := version._cand.file_list[0][0]:
		if origin.component == "now":
			return _("{pkg} [local]").format(pkg=line)
		return f"{line} [{origin.label}/{origin.codename} {origin.component}]"
	return line


def set_search_installed(
	line: str, top_line: str, pkg: Package, version: Version
) -> str:
	"""Return the provided string with install and upgrade information."""
	if version.is_installed and pkg.is_upgradable:
		return _(
			"{pkg_name}\n{tree_start} is installed and upgradable from {version}"
		).format(
			pkg_name=line,
			tree_start=top_line,
			version=color(pkg_installed(pkg).version, "BLUE"),
		)
	if pkg.is_upgradable:
		return _("{pkg_name}\n{tree_start} is upgradable from {version}").format(
			pkg_name=line,
			tree_start=top_line,
			version=color(pkg_installed(pkg).version, "BLUE"),
		)
	if version.is_installed:
		return _("{pkg_name}\n{tree_start} is installed").format(
			pkg_name=line, tree_start=top_line
		)
	return line


def set_search_description(line: str, bot_line: str, version: Version) -> str:
	"""Return the provided string with the package description."""
	if arguments.full and version._translated_records:
		desc = "\n    ".join(version._translated_records.long_desc.splitlines())
		return f"{line}\n{bot_line} {desc}"
	if version.summary:
		return f"{line}\n{bot_line} {version.summary}"
	no_desc = _("No Description")
	return f"{line}\n{bot_line}{COLOR_CODES['ITALIC']} {no_desc}{COLOR_CODES['RESET']}"
