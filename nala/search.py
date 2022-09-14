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
# nala is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# nala is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with nala.  If not, see <https://www.gnu.org/licenses/>.
"""Functions for the Nala Search command."""
from __future__ import annotations

from typing import Iterable, Pattern

from apt import Cache, Package, Version

from nala import COLOR_CODES, _, color
from nala.options import arguments
from nala.rich import ascii_replace, is_utf8
from nala.utils import get_version, pkg_candidate, pkg_installed

TOP_LINE = "├──" if is_utf8 else "+--"
BOT_LINE = "└──" if is_utf8 else "`--"
LINE = "│   " if is_utf8 else "|   "


def skip_pkg(
	cache: Cache, pkg: Package, nala_installed: bool, user_installed: list[str]
) -> bool:
	"""Check if the package should be skipped based on user switches."""
	if nala_installed and pkg.shortname not in user_installed:
		return True
	if arguments.installed and not pkg.installed:
		return True
	if arguments.upgradable and not pkg.is_upgradable:
		return True
	return bool(arguments.virtual and not cache.is_virtual_package(pkg.name))


def search_name(pkg: Package, patterns: list[Pattern[str]]) -> bool:
	"""Search the package name and description."""
	searches = [pkg.shortname]
	if not arguments.names:
		records = pkg._pcache._records
		records.lookup(pkg._pkg.version_list[0].file_list[0])
		searches.extend([records.long_desc, records.source_pkg])

	for pattern in patterns:
		for string in searches:
			if pattern.search(string):
				return True
	return False


def iter_search(found: Iterable[Package]) -> bool:
	"""Iterate the search results."""
	pkg_list_check: list[Package] = []
	for pkg in found:
		version = get_version(pkg, inst_first=True)
		# Version being a tuple here means --all-versions has been enabled
		if isinstance(version, tuple):
			for ver in version:
				print_search(pkg, ver, pkg_list_check)
		else:
			print_search(pkg, version, pkg_list_check)

	return bool(pkg_list_check)


def print_search(pkg: Package, version: Version, pkg_list_check: list[Package]) -> None:
	"""Print the search results to the terminal."""
	pkg_list_check.append(pkg)
	print(
		ascii_replace(
			set_search_description(
				set_search_installed(
					set_search_origin(
						f"{color(pkg.name, 'GREEN')} {color(version.version, 'BLUE')}",
						version,
					),
					pkg,
					version,
				),
				version,
			)
		),
		end="\n\n",
	)


def set_search_origin(line: str, version: Version) -> str:
	"""Return the provided string with the origin information."""
	if origin := version._cand.file_list[0][0]:
		if origin.component == "now":
			return _("{package} [local]").format(package=line)
		return f"{line} [{origin.label}/{origin.codename} {origin.component}]"
	return line


def set_search_installed(line: str, pkg: Package, version: Version) -> str:
	"""Return the provided string with install and upgrade information."""
	if version.is_installed and pkg.is_upgradable:
		# NOTE: Formatting looks as below:
		# NOTE: vim 2:8.2.3995-1+b2 [Debian/sid main]
		# NOTE: ├── is installed and upgradable to 2:8.2.4659-1
		# NOTE: └── Vi IMproved - enhanced vi editor
		return f"{line}\n{TOP_LINE} " + _(
			"is installed and upgradable to {version}"
		).format(
			version=color(pkg_candidate(pkg).version, "BLUE"),
		)
	if version == pkg.candidate and pkg.is_upgradable:
		# NOTE: vim 2:8.2.4659-1 [Debian/sid main]
		# NOTE: ├── is upgradable from 2:8.2.3995-1+b2
		# NOTE: └── Vi IMproved - enhanced vi editor
		return f"{line}\n{TOP_LINE} " + _("is upgradable from {version}").format(
			version=color(pkg_installed(pkg).version, "BLUE")
		)
	if version.is_installed:
		# NOTE: vim 2:8.2.3995-1+b2 [Debian/sid main]
		# NOTE: ├── is installed
		# NOTE: └── Vi IMproved - enhanced vi editor
		return f"{line}\n{TOP_LINE} " + _("is installed")
	return line


def set_search_description(line: str, version: Version) -> str:
	"""Return the provided string with the package description."""
	records = version._translated_records
	if arguments.full and records:
		desc = "\n    ".join(records.long_desc.splitlines())
		return f"{line}\n{BOT_LINE} {desc}"
	if records:
		return f"{line}\n{BOT_LINE} {records.short_desc}"
	# NOTE: vim 2:8.2.3995-1+b2 [Debian/sid main]
	# NOTE: ├── is installed
	# NOTE: └── No Description
	no_desc = _("No Description")
	return f"{line}\n{BOT_LINE}{COLOR_CODES['ITALIC']} {no_desc}{COLOR_CODES['RESET']}"
