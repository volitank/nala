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
"""Where Utilities who don't have a special home come together."""
from __future__ import annotations

import sys
from pathlib import Path

from pyshell import pyshell

from nala.constants import COLOR_CODES, COLUMNS, ERROR_PREFIX
from nala.logger import dprint
from nala.rich import Table, console

shell = pyshell(capture_output=True, text=True, check=True)

def color(text: str, text_color: str = 'WHITE') -> str:
	"""Return bold text in the color of your choice."""
	return f'\x1b[1;{COLOR_CODES[text_color]}m' + text + str(COLOR_CODES['RESET'])

def dir_check(path: Path, msg: str) -> None:
	"""Check to see if the directory exists in apt config."""
	if not path:
		sys.exit(ERROR_PREFIX+msg)

def ask(question: str, default_no: bool = False) -> bool:
	"""Ask the user {question}.

	resp = input(f'{question}? [Y/n]

	Y returns True
	N returns False
	"""
	while True:
		resp = input(f'{question}? [Y/n] ')
		if resp in ('y', 'Y'):
			return True
		if resp in ('n', 'N'):
			return False
		if resp == '':
			return not default_no
		print("Not a valid choice kiddo")

def unit_str(val: int, just: int = 7) -> str:
	"""Check integer and figure out what format it should be."""
	if val > 1000**2:
		return f"{val/1000/1000 :.1f}".rjust(just)+" MB"
	if val > 1000:
		return f"{round(val/1000) :.0f}".rjust(just)+" kB"
	return f'{val :.0f}'.rjust(just)+" B"

def iter_remove(path: Path, verbose: bool = False) -> None:
	"""Iterate the directory supplied and remove all files."""
	if verbose:
		print(f'Removing files in {path}')
	for file in path.iterdir():
		if file.is_file():
			dprint(f'Removed: {file}')
			file.unlink(missing_ok=True)

def print_packages(
	headers: list[str], names: list[list[str]],
	title: str, style: str | None = None) -> None:
	"""Print package transactions in a pretty format."""
	if not names:
		return

	package_table = Table(padding=(0,2), box=None)
	# Setup rich table and columns
	for header in headers:
		if header == 'Package:':
			package_table.add_column(header, style=style)
		elif header == 'Size:':
			package_table.add_column(header, justify='right')
		else:
			package_table.add_column(header)

	# Get the size and change it to size
	for package in names:
		size = package.pop()
		package.append(unit_str(int(size)))

	# Add our packages
	for name in names:
		package_table.add_row(*name)

	sep = '='*COLUMNS
	console.print(
		sep,
		title,
		sep,
		package_table)
