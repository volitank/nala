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

from nala.constants import ERROR_PREFIX
from nala.utils import color, term
from nala.rich import Columns, Text
from nala.utils import get_installed_dep_names, print_rdeps
from nala.show import print_dep

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
