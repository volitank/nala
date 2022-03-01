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
"""Functions for Nala errors."""
from __future__ import annotations

import sys
from typing import NoReturn, cast

import apt_pkg
from apt.cache import FetchFailedException, LockFailedException
from apt.package import Package, Version

from nala.constants import ERROR_PREFIX, _
from nala.rich import Columns, Text
from nala.show import SHOW_INFO, show_dep
from nala.utils import (color, eprint,
				get_installed_dep_names, print_rdeps, term)


class ExitCode: # pylint: disable=too-few-public-methods
	"""Constants for Exit Codes."""

	SIGINT = 130
	SIGTERM = 143

def apt_error(apt_err: FetchFailedException | LockFailedException | apt_pkg.Error) -> NoReturn:
	"""Take an error message from python-apt and formats it."""
	msg = str(apt_err)
	if not msg:
		# Sometimes python apt gives us literally nothing to work with.
		# Probably an issue with sources.list. Needs further testing.
		sys.exit(
			_("{error} python-apt gave us {apt_err} This isn't a proper error as it's empty").format(
				error=ERROR_PREFIX, apt_err=repr(apt_err)
			)
			)
	if ',' in msg:
		err_list = set(msg.split(','))
		for err in err_list:
			if 'E:' in err:
				err = err.replace('E:', '')
				eprint(f"{ERROR_PREFIX} {err.strip()}")
				continue
			if 'W:' in err:
				err = err.replace('W:', '')
				warn = color(_('Warning:'), 'YELLOW')
				eprint(f"{warn} {err.strip()}")
				continue
		sys.exit(1)
	eprint(ERROR_PREFIX+msg)
	if not term.is_su():
		sys.exit(_('Are you root?'))
	sys.exit(1)

def essential_error(pkg_list: list[Text]) -> NoReturn:
	"""Print error message for essential packages and exit."""
	print('='*term.columns)
	print(
		_("The Following Packages are {essential}").format(
			essential=color('Essential!', 'RED')
		)
	)
	print('='*term.columns)
	term.console.print(Columns(pkg_list, padding=(0,2), equal=True))
	print('='*term.columns)
	eprint(
		_("{error} You have attempted to remove {essential}").format(
			error=ERROR_PREFIX, essential=color('essential packages', 'RED')
		)
	)
	eprint(
		_("{error} Please use {switch} if you are sure you want to.").format(
			error=ERROR_PREFIX, switch=color('--remove-essential', 'YELLOW')
		)
	)
	sys.exit(1)

def pkg_error(pkg_list: list[str], terminate: bool = False) -> None:
	"""Print error for package in list."""
	for pkg in pkg_list:
		eprint(
			_("{error} {pkg} not found").format(
				error=ERROR_PREFIX, pkg=color(pkg, 'YELLOW')
			)
		)
	if terminate:
		sys.exit(1)

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
	print(
		_("\n{notice} The information above may be able to help").format(
			notice=color(_('Notice:'), 'YELLOW')
		)
	)
	sys.exit(
		_("{error} You have held broken packages").format(
			error=ERROR_PREFIX
		)
	)

def print_broken(pkg_name: str, candidate: Version) -> None:
	"""Print broken packages information."""
	print('='*term.columns)
	version = color('(') + color(candidate.version, 'BLUE') + color(')')
	print(f"{color('Package:', 'YELLOW')} {color(pkg_name, 'GREEN')} {version}")
	msg = ''
	if conflicts := candidate.get_dependencies('Conflicts'):
		msg += SHOW_INFO.format(
			header=color(_('Conflicts:'), 'YELLOW'),
			info=show_dep(conflicts)
		)
	if breaks := candidate.get_dependencies('Breaks'):
		msg += SHOW_INFO.format(
			header=color(_('Breaks:'), 'YELLOW'),
			info=show_dep(breaks)
		)
	if candidate.dependencies:
		msg += SHOW_INFO.format(
			header=color(_('Depends:'), 'YELLOW'),
			info=show_dep(candidate.dependencies)
		)
	print(msg)
