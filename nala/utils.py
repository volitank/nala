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

import contextlib
import json
import os
import signal
import sys
import termios
import tty
from dataclasses import dataclass, field
from datetime import datetime
from fcntl import LOCK_EX, LOCK_NB, lockf
from pathlib import Path
from types import FrameType
from typing import TYPE_CHECKING, Any, Iterable

import jsbeautifier
from apt.package import Package, Version

from nala import _, color, console
from nala.constants import (
	ERROR_PREFIX,
	HANDLER,
	JSON_OPTIONS,
	NALA_DEBUGLOG,
	NALA_LOCK_FILE,
)
from nala.options import arguments
from nala.rich import from_ansi

if TYPE_CHECKING:
	from nala.debfile import NalaDebPackage

LOCK_FILE = None

# NOTE: Answers for the Question prompt "[Y/n]"
YES_NO = _("Y/y N/n").split()
YES = YES_NO[0].split("/")
NO = YES_NO[1].split("/")


class Terminal:
	"""Represent the user terminal."""

	# Term Constants
	STDIN = 0
	STDOUT = 1
	STDERR = 2

	# Control Codes
	CURSER_UP = b"\x1b[1A"
	CLEAR_LINE = b"\x1b[2k"
	CLEAR = b"\x1b[2J"
	CLEAR_FROM_CURRENT_TO_END = b"\x1b[K"
	BACKSPACE = b"\x08"
	HOME = b"\x1b[H"
	ENABLE_BRACKETED_PASTE = b"\x1b[?2004h"
	DISABLE_BRACKETED_PASTE = b"\x1b[?2004l"
	ENABLE_ALT_SCREEN = b"\x1b[?1049h"
	DISABLE_ALT_SCREEN = b"\x1b[?1049l"
	SHOW_CURSOR = b"\x1b[?25h"
	HIDE_CURSOR = b"\x1b[?25l"
	SET_CURSER = b"\x1b[?1l"
	SAVE_TERM = b"\x1b[22;0;0t"
	RESTORE_TERM = b"\x1b[23;0;0t"
	APPLICATION_KEYPAD = b"\x1b="
	NORMAL_KEYPAD = b"\x1b>"
	CR = b"\r"
	LF = b"\n"
	CRLF = b"\r\n"

	def __init__(self) -> None:
		"""Represent the user terminal."""
		self.console = console
		self.mode: list[int | list[bytes | int]] = []
		self.term_type: str = os.environ.get("TERM", "")
		self.locale: str = ""
		self.set_environment()

	def __repr__(self) -> str:
		"""Represent state of the user terminal as a string."""
		representation = {
			"object": "Terminal",
			"columns": self.columns,
			"lines": self.lines,
			"mode": str(self.mode),
			"term": self.console.is_terminal,
		}
		return str(jsbeautifier.beautify(json.dumps(representation), JSON_OPTIONS))

	def set_environment(self) -> None:
		"""Check and set various environment variables."""
		# Termios can't run if we aren't in a terminal
		# Just catch the exception and continue.
		with contextlib.suppress(termios.error):
			self.mode = termios.tcgetattr(self.STDIN)
		if self.lines < 13 or self.columns < 31:
			print(
				_("Terminal can't support dialog, falling back to readline"),
				file=sys.stderr,
			)
			os.environ["DEBIAN_FRONTEND"] = "readline"
		# Readline is too hard to support with our fancy formatting
		if os.environ.get("DEBIAN_FRONTEND") == "readline":
			arguments.raw_dpkg = True
		os.environ["DPKG_COLORS"] = "never"
		self.locale = os.environ.get("LANG", "")
		# We have to set lang as C so we get predictable output from dpkg.
		os.environ["LANG"] = "C" if self.console.options.ascii_only else "C.UTF-8"

	@property
	def columns(self) -> int:
		"""Return termindal width."""
		return self.console.width

	@property
	def lines(self) -> int:
		"""Return terminal height."""
		return self.console.height

	def restore_mode(self) -> None:
		"""Restore the mode the Terminal was initialized with."""
		if not self.console.is_terminal:
			return
		with contextlib.suppress(termios.error):
			termios.tcsetattr(self.STDIN, termios.TCSAFLUSH, self.mode)

	def restore_locale(self) -> None:
		"""Restore the locale to it's original value."""
		os.environ["LANG"] = self.locale

	def set_raw(self) -> None:
		"""Set terminal raw."""
		with contextlib.suppress(termios.error):
			tty.setraw(self.STDIN)

	def write(self, data: bytes) -> None:
		"""Write bytes directly to stdout."""
		os.write(self.STDOUT, data)

	def is_xterm(self) -> bool:
		"""Return True if we're in an xterm, False otherwise."""
		return "xterm" in self.term_type

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
		dprint("SIGINT received. Delaying KeyboardInterrupt.")

	def __exit__(self, _type: None, _value: None, _traceback: None) -> None:
		"""Exit context."""
		signal.signal(signal.SIGINT, self.old_handler)
		if isinstance(self.signal_received, tuple) and callable(self.old_handler):
			self.old_handler(*self.signal_received)


@dataclass
class PackageHandler:  # pylint: disable=too-many-instance-attributes
	"""Class for storing package lists."""

	autoremoved: list[str] = field(default_factory=list)
	user_explicit: list[Package] = field(default_factory=list)
	local_debs: list[NalaDebPackage] = field(default_factory=list)
	delete_pkgs: list[NalaPackage] = field(default_factory=list)
	install_pkgs: list[NalaPackage] = field(default_factory=list)
	reinstall_pkgs: list[NalaPackage] = field(default_factory=list)
	upgrade_pkgs: list[NalaPackage] = field(default_factory=list)
	autoremove_pkgs: list[NalaPackage] = field(default_factory=list)
	recommend_pkgs: list[NalaPackage | list[NalaPackage]] = field(default_factory=list)
	suggest_pkgs: list[NalaPackage | list[NalaPackage]] = field(default_factory=list)
	configure_pkgs: list[NalaPackage] = field(default_factory=list)
	downgrade_pkgs: list[NalaPackage] = field(default_factory=list)

	def dpkg_progress_total(self) -> int:
		"""Calculate our total operations for the dpkg progress bar."""
		return (
			len(self.delete_pkgs) * 2
			+ len(self.autoremove_pkgs) * 2
			# We add an extra for each install due to Unpacking: and Setting up:
			+ len(self.install_pkgs) * 2
			+ len(self.reinstall_pkgs) * 2
			+ len(self.downgrade_pkgs) * 2
			+ len(self.upgrade_pkgs) * 2
			# For local deb installs we add 1 more because of having to start
			# and stop InstallProgress an extra time for each package
			+ len(self.local_debs)
			# Configure needs an extra because it isn't unpacked
			+ len(self.configure_pkgs) * 2
			# This last +1 for the ending of dpkg itself
			+ 1
		)


@dataclass
class NalaPackage:
	"""Class that represents a Nala package."""

	name: str
	version: str
	size: int
	old_version: str | None = None

	@property
	def unit_size(self) -> str:
		"""Return the size as a readable unit. Example 12MB."""
		return unit_str(self.size)


term = Terminal()


def ask(question: str, default_no: bool = False) -> bool:
	"""Ask the user {question}.

	resp = input(f'{question}? [Y/n]

	Y returns True
	N returns False
	"""
	if arguments.assume_yes:
		return True
	if arguments.assume_no:
		return False
	while True:
		resp = input(f"{question} [{YES[0]}/{NO[1]}] ")
		if resp in YES:
			return True
		if resp in NO:
			return False
		if resp == "":
			return not default_no
		print(_("Not a valid choice kiddo"))


def sudo_check(args: Iterable[str] | None = None) -> None:
	"""Check for root and exit if not root."""
	if not term.is_su():
		if arguments.command == "install" and arguments.fix_broken and not args:
			sys.exit(
				_("{error} Nala needs root to fix broken packages").format(
					error=ERROR_PREFIX
				)
			)
		sys.exit(
			_("{error} Nala needs root to {command}").format(
				error=ERROR_PREFIX, command=arguments.command
			)
		)

	NALA_LOCK_FILE.touch(exist_ok=True)
	global LOCK_FILE  # pylint: disable=global-statement
	LOCK_FILE = NALA_LOCK_FILE.open("r+", encoding="ascii")
	current_pid = os.getpid()
	last_pid = LOCK_FILE.read()

	try:
		dprint("Setting Lock")
		lockf(LOCK_FILE, LOCK_EX | LOCK_NB)
		LOCK_FILE.seek(0)
		LOCK_FILE.write(f"{current_pid}")
		LOCK_FILE.truncate()
		dprint("Lock Set")
	except OSError:
		sys.exit(
			_("{error} Nala is already running another instance {last_pid}").format(
				error=ERROR_PREFIX, last_pid=color(last_pid, "YELLOW")
			)
		)


def get_date() -> str:
	"""Return the formatted Date and Time."""
	timezone = str(datetime.utcnow().astimezone().tzinfo)
	return f"{datetime.now().strftime('%Y-%m-%d %H:%M:%S')} {timezone}"


def unit_str(val: int, just: int = 7) -> str:
	"""Check integer and figure out what format it should be."""
	if val > 1000**2:
		return f"{val/1000/1000 :.1f}".rjust(just) + " MB"
	if val > 1000:
		return f"{round(val/1000) :.0f}".rjust(just) + " kB"
	return f"{val :.0f}".rjust(just) + " B"


def iter_remove(path: Path) -> None:
	"""Iterate the directory supplied and remove all files."""
	vprint(_("Removing files in {dir}").format(dir=path))
	for file in path.iterdir():
		if file.is_file():
			vprint(_("Removed: {filename}").format(filename=file))
			file.unlink(missing_ok=True)


def get_version(
	pkg: Package, cand_first: bool = False, inst_first: bool = False
) -> Version | tuple[Version, ...]:
	"""Get the version, any version of a package."""
	if not cand_first and arguments.all_versions:
		return tuple(pkg.versions)
	if cand_first:
		return pkg.candidate or pkg.installed or pkg.versions[0]
	if inst_first:
		return pkg.installed or pkg.candidate or pkg.versions[0]
	for version in pkg.versions:
		return version
	# It would be really weird if we ever actually hit this error
	sys.exit(
		_("{error} can't find version for {package}").format(
			error=ERROR_PREFIX, package=pkg.name
		)
	)


def get_pkg_name(candidate: Version) -> str:
	"""Return the package name.

	Checks if we need and epoch in the path.
	"""
	if ":" in candidate.version:
		index = candidate.version.index(":")
		epoch = f"_{candidate.version[:index]}%3a"
		return Path(candidate.filename).name.replace("_", epoch, 1)
	return Path(candidate.filename).name


def pkg_candidate(pkg: Package) -> Version:
	"""Type enforce package candidate."""
	assert pkg.candidate
	return pkg.candidate


def pkg_installed(pkg: Package) -> Version:
	"""Type enforce package installed."""
	assert pkg.installed
	return pkg.installed


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


def print_rdeps(name: str, installed_pkgs: tuple[Package]) -> None:
	"""Print the installed reverse depends of a package."""
	msg = color(
		_("Installed packages that depend on {package}").format(
			package=color(name, "GREEN")
		)
		+ "\n",
		"YELLOW",
	)
	for pkg in installed_pkgs:
		for dep in pkg_installed(pkg).dependencies:
			if name in dep.rawstr:
				dep_msg = f"  {color(pkg.name, 'GREEN')}"
				if pkg.essential:
					dep_msg = _("{package} is an Essential package!").format(
						package=dep_msg
					)
				msg += f"{dep_msg}\n"
				break
	print(msg.strip())


def dedupe_list(original: Iterable[str]) -> list[str]:
	"""Deduplicate a list.

	Useful for when we want to maintain the list order and can't use set()
	"""
	dedupe = []
	for item in original:
		if item not in dedupe:
			dedupe.append(item)
	return dedupe


def vprint(msg: object) -> None:
	"""Print message if verbose."""
	msg = str(msg)
	if arguments.verbose or arguments.debug:
		print(msg)
	if arguments.debug:
		dprint(from_ansi(msg).plain, from_verbose=True)
	sys.__stdout__.flush()


def dprint(msg: object, from_verbose: bool = False) -> None:
	"""Print message if debugging, write to log if root."""
	if not arguments.debug:
		return
	if not from_verbose:
		print(f"DEBUG: {msg}")
	if term.is_su():
		with open(NALA_DEBUGLOG, "a", encoding="utf-8") as logfile:
			logfile.write(f"[{get_date()}] DEBUG: {msg}\n")


def eprint(*args: Any, **kwargs: Any) -> None:
	"""Print message to stderr."""
	print(*args, file=sys.stderr, **kwargs)
