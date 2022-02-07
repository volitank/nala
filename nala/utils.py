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

import hashlib
import json
import os
import signal
import sys
import termios
import tty
from datetime import datetime
from pathlib import Path
from shutil import get_terminal_size
from types import FrameType

import jsbeautifier
from apt.package import Package, Version

from nala.constants import (COLOR_CODES, ERROR_PREFIX,
				HANDLER, JSON_OPTIONS, NALA_DEBUGLOG)
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

class DelayedKeyboardInterrupt:
	"""Context manager to delay KeyboardInterrupt.

	Keyboard Interrupts will be delayed until out of scope.
	"""

	def __init__(self) -> None:
		"""Context manager to delay KeyboardInterrupt."""
		self.signal_received: tuple[int, FrameType | None] | bool
		self.old_handler: HANDLER

	def __enter__(self) -> None:
		"""Enter context."""
		self.signal_received = False
		self.old_handler = signal.signal(signal.SIGINT, self.handler)

	def handler(self, sig: int, frame: FrameType | None) -> None:
		"""Handle sigint signals."""
		self.signal_received = (sig, frame)
		dprint('SIGINT received. Delaying KeyboardInterrupt.')

	def __exit__(self, _type: None, _value: None, _traceback: None) -> None:
		"""Exit context."""
		signal.signal(signal.SIGINT, self.old_handler)
		if isinstance(self.signal_received, tuple) and callable(self.old_handler):
			self.old_handler(*self.signal_received)

term = Terminal()

def color(text: str, text_color: str = 'WHITE') -> str:
	"""Return bold text in the color of your choice."""
	return f'\x1b[1;{COLOR_CODES[text_color]}m' + text + str(COLOR_CODES['RESET'])

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

def check_pkg(directory: Path, candidate: Package | Version) -> bool:
	"""Check if file exists, is correct, and run check hash."""
	if isinstance(candidate, Package):
		candidate = pkg_candidate(candidate)
	path = directory / get_pkg_name(candidate)
	if not path.exists() or path.stat().st_size != candidate.size:
		return False
	hash_type, hash_value = get_hash(candidate)
	try:
		return check_hash(path, hash_type, hash_value)
	except OSError as err:
		print("Failed to check hash", err)
		return False

def check_hash(path: Path, hash_type: str, hash_value: str) -> bool:
	"""Check hash value."""
	hash_fun = hashlib.new(hash_type)
	with path.open('rb') as file:
		while True:
			data = file.read(4096)
			if not data:
				break
			hash_fun.update(data)
	local_hash = hash_fun.hexdigest()
	debugger = (
		str(path),
		f"Candidate Hash: {hash_type} {hash_value}",
		f"Local Hash: {local_hash}"
		f"Hash Success: {local_hash == hash_value}"
	)
	dprint(debugger)
	return local_hash == hash_value

def get_hash(version: Version) -> tuple[str, str]:
	"""Get the correct hash value."""
	if version.sha256:
		return ("sha256", version.sha256)
	if version.sha1:
		return ("sha1", version.sha1)
	if version.md5:
		return ("md5", version.md5)
	sys.exit(ERROR_PREFIX+f"{Path(version.filename).name} can't be checked for integrity.")

def get_pkg_name(candidate: Version) -> str:
	"""Return the package name.

	Checks if we need and epoch in the path.
	"""
	if ':' in candidate.version:
		index = candidate.version.index(':')
		epoch = '_'+candidate.version[:index]+r'%3a'
		return Path(candidate.filename).name.replace('_', epoch, 1)
	return Path(candidate.filename).name

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

def vprint(msg: str) -> None:
	"""Print message if verbose."""
	if arguments.verbose:
		print(msg)

def dprint(msg: object) -> None:
	"""Print message if debugging, write to log if root."""
	if arguments.debug:
		msg = str(msg)
		print('DEBUG: '+msg)
		if term.is_su():
			timezone = datetime.utcnow().astimezone().tzinfo
			time = datetime.now().strftime('%Y-%m-%d %H:%M:%S')+' '+str(timezone)
			msg = f'[{time}] DEBUG: ' + msg
			with open(NALA_DEBUGLOG, 'a', encoding='utf-8') as logfile:
				logfile.write(msg+'\n')
