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

import json
import os
import sys
import termios
import tty
from pathlib import Path
from shutil import get_terminal_size

import jsbeautifier
from apt.package import Package, Version

from nala.constants import COLOR_CODES, ERROR_PREFIX, JSON_OPTIONS
from nala.logger import dprint
from nala.options import arguments
from nala.rich import Table, console


class Terminal:
	"""Represent the user terminal."""

	# Term Constants
	STDIN = 0
	STDOUT = 1
	STDERR = 2

	# Control Codes
	CURSER_UP = b'\x1b[1A'
	CLEAR_LINE = b'\x1b[2k'
	CLEAR = b'\x1b[2J'
	CLEAR_FROM_CURRENT_TO_END = b'\x1b[K'
	BACKSPACE = b'\x08'
	HOME = b'\x1b[H'
	ENABLE_BRACKETED_PASTE = b'\x1b[?2004h'
	DISABLE_BRACKETED_PASTE = b'\x1b[?2004l'
	ENABLE_ALT_SCREEN = b'\x1b[?1049h'
	DISABLE_ALT_SCREEN = b'\x1b[?1049l'
	SHOW_CURSOR = b'\x1b[?25h'
	HIDE_CURSOR = b'\x1b[?25l'
	SET_CURSER = b'\x1b[?1l'
	SAVE_TERM = b'\x1b[22;0;0t'
	RESTORE_TERM = b'\x1b[23;0;0t'
	APPLICATION_KEYPAD = b'\x1b='
	NORMAL_KEYPAD = b'\x1b>'
	CR = b'\r'
	LF = b'\n'
	CRLF = b'\r\n'

	def __init__(self) -> None:
		"""Represent the user terminal."""
		self.size = get_terminal_size()
		self.columns = self.size.columns
		self.lines = self.size.lines
		self.console = console
		self.term: bool = True
		self.mode: list[int | list[bytes | int]] = []
		self.term_type: str = os.environ.get('TERM', '')
		self.check()
		if self.lines < 13 or self.columns < 31:
			print("Terminal can't support dialog, falling back to readline")
			os.environ["DEBIAN_FRONTEND"] = "readline"
		# Readline is too hard to support with our fancy formatting
		if os.environ.get("DEBIAN_FRONTEND") == "readline":
			arguments.raw_dpkg = True

	def __repr__(self) -> str:
		"""Represent state of the user terminal as a string."""
		representation = {
			'object' : 'Terminal',
			'size' : self.size,
			'columns' : self.size.columns,
			'lines' : self.lines,
			'mode'	: str(self.mode),
			'term' : self.term
		}
		return str(jsbeautifier.beautify(json.dumps(representation), JSON_OPTIONS))

	def check(self) -> None:
		"""Check if we are a terminal or piped."""
		self.term = bool(sys.stdout.isatty())
		if self.term:
			# There are some cases where we need extra checks
			# For example whatever hyperfine is doing with --show-output
			try:
				self.mode = termios.tcgetattr(self.STDIN)
				self.term = True
			# We catch and handle 'Inappropriate ioctl for device'.
			except termios.error as err:
				# And then we set term off.
				if err.args[0] == 25:
					self.term = False
				else:
					sys.exit(err)

	def update_size(self) -> None:
		"""Update the current width and length of the terminal."""
		self.size = get_terminal_size()
		self.columns = self.size.columns
		self.lines = self.size.lines

	def restore_mode(self) -> None:
		"""Restore the mode the Terminal was initialized with."""
		if self.term:
			termios.tcsetattr(self.STDIN, termios.TCSAFLUSH, self.mode)

	def set_raw(self) -> None:
		"""Set terminal raw."""
		if self.term:
			tty.setraw(self.STDIN)

	def write(self, data: bytes) -> None:
		"""Write bytes directly to stdout."""
		os.write(self.STDOUT, data)

	def is_term(self) -> bool:
		"""Return true if we are a terminal. False if piped."""
		return self.term

	def is_xterm(self) -> bool:
		"""Return True if we're in an xterm, False otherwise."""
		return 'xterm' in self.term_type

	@staticmethod
	def is_su() -> bool:
		"""Return True if we're super user and False if we're not."""
		return os.geteuid() == 0

term = Terminal()

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

def pkg_candidate(pkg: Package) -> Version:
	"""Type enforce package candidate."""
	assert pkg.candidate
	return pkg.candidate

def pkg_installed(pkg: Package) -> Version:
	"""Type enforce package installed."""
	assert pkg.installed
	return pkg.installed

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

	sep = '='*term.columns
	console.print(
		sep,
		title,
		sep,
		package_table)

def verbose_print(msg: str) -> None:
	"""Print message if verbose."""
	if arguments.verbose:
		print(msg)
