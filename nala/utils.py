#                 __
#    ____ _____  |  | _____
#   /    \\__  \ |  | \__  \
#  |   |  \/ __ \|  |__/ __ \_
#  |___|  (____  /____(____  /
#       \/     \/          \/
#
# Copyright (C) 2010 - 2021 Tatsuhiro Tsujikawa
# Copyright (C) 2021, 2022 Blake Lee
#
# This file is part of nala
# nala is based upon apt-metalink https://github.com/tatsuhiro-t/apt-metalink
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
"""Where Utilities who don't have a special home come together."""
from __future__ import annotations

import contextlib
import os
import re
import signal
import sys
import termios
import tty
from dataclasses import dataclass, field
from datetime import datetime
from fcntl import LOCK_EX, LOCK_NB, lockf
from pathlib import Path
from types import FrameType
from typing import TYPE_CHECKING, Any, Generator, Iterable, Pattern

from apt.package import Package, Version

from nala import TERMUX, _, color, console
from nala.constants import (
	ERROR_PREFIX,
	HANDLER,
	NALA_DEBUGLOG,
	NALA_DIR,
	NALA_LOCK_FILE,
	NALA_LOGDIR,
	NOTICE_PREFIX,
)
from nala.options import arguments
from nala.rich import from_ansi

if TYPE_CHECKING:
	from nala.debfile import NalaDebPackage
	from nala.fetch import FetchLive

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
	CURSER_DOWN = b"\x1b[1B"
	CURSER_FORWARD = b"\x1b[1C"
	CURSER_BACK = b"\x1b[1D"
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
		self.term_type: str = os.environ.get("TERM", "").lower()
		self.locale: str = ""
		self.set_environment()

	def __repr__(self) -> str:
		"""Represent state of the user terminal as a string."""
		kwarg = "\n    ".join(
			(f"{key} = {value},") for key, value in self.__dict__.items()
		)
		return f"Terminal = [\n    {kwarg}\n]"

	def set_environment(self) -> None:
		"""Check and set various environment variables."""
		# Termios can't run if we aren't in a terminal
		# Just catch the exception and continue.
		if self.can_format():
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

	def can_format(self) -> bool:
		"""Return if we're allowed to do anything fancy."""
		return (
			os.isatty(self.STDOUT)
			and os.isatty(self.STDIN)
			and self.term_type not in ("dumb", "unknown")
		)

	def restore_mode(self) -> None:
		"""Restore the mode the Terminal was initialized with."""
		if self.can_format():
			termios.tcsetattr(self.STDIN, termios.TCSAFLUSH, self.mode)

	def restore_locale(self) -> None:
		"""Restore the locale to it's original value."""
		os.environ["LANG"] = self.locale

	def set_raw(self) -> None:
		"""Set terminal raw."""
		if self.can_format():
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
		return TERMUX or os.geteuid() == 0


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

	autoremoved: set[str] = field(default_factory=set)
	user_explicit: list[Package] = field(default_factory=list)
	local_debs: list[NalaDebPackage] = field(default_factory=list)
	# Packages that can be auto removed, but won't
	not_needed: list[NalaPackage] = field(default_factory=list)
	delete_pkgs: list[NalaPackage] = field(default_factory=list)
	install_pkgs: list[NalaPackage] = field(default_factory=list)
	reinstall_pkgs: list[NalaPackage] = field(default_factory=list)
	upgrade_pkgs: list[NalaPackage] = field(default_factory=list)
	autoremove_pkgs: list[NalaPackage] = field(default_factory=list)
	autoremove_config: list[NalaPackage] = field(default_factory=list)
	delete_config: list[NalaPackage] = field(default_factory=list)
	recommend_pkgs: list[NalaPackage | list[NalaPackage]] = field(default_factory=list)
	suggest_pkgs: list[NalaPackage | list[NalaPackage]] = field(default_factory=list)
	configure_pkgs: list[NalaPackage] = field(default_factory=list)
	downgrade_pkgs: list[NalaPackage] = field(default_factory=list)
	held_pkgs: list[NalaPackage] = field(default_factory=list)

	def no_summary(
		self, pkg_set: list[NalaPackage] | list[NalaPackage | list[NalaPackage]]
	) -> bool:
		"""Return True if we shouldn't print a summary for the package set."""
		return pkg_set in (self.suggest_pkgs, self.recommend_pkgs, self.not_needed)

	def all_pkgs(self) -> Generator[NalaPackage | NalaDebPackage, None, None]:
		"""Return a list of all the packages to be altered."""
		yield from (
			self.delete_pkgs
			+ self.autoremove_pkgs
			+ self.install_pkgs
			+ self.reinstall_pkgs
			+ self.downgrade_pkgs
			+ self.upgrade_pkgs
			+ self.configure_pkgs
			+ self.autoremove_config
			+ self.delete_config
		)
		yield from self.local_debs

	def dpkg_progress_total(self) -> int:
		"""Calculate our total operations for the dpkg progress bar."""
		return (
			len(
				self.delete_pkgs
				+ self.autoremove_pkgs
				+ self.install_pkgs
				+ self.reinstall_pkgs
				+ self.downgrade_pkgs
				+ self.upgrade_pkgs
				+ self.configure_pkgs
			)
			* 2
			# For local deb installs we add 1 more because of having to start
			# and stop InstallProgress an extra time for each package
			+ len(self.local_debs)
			# Purging configuration files only have 1 message
			+ len(self.autoremove_config + self.delete_config)
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
		"""Return the size as a readable unit. Example 12 MB."""
		return unit_str(self.size)


term = Terminal()


def command_help(wrong: str, correct: str, update: bool | None) -> None:
	"""Check if the user typed a common mistake of a command."""
	command_ask = False
	if arguments.command == "history" and arguments.history == wrong:
		arguments.command = correct
		wrong = f"history {wrong}"
		command_ask = True

	elif arguments.command == wrong:
		arguments.command = correct
		command_ask = True

	if command_ask:
		arguments.set_update(update)
		if not ask(
			_("{command} is not a command\nDid you mean {correction}?").format(
				command=color(wrong, "YELLOW"),
				correction=color(f"nala {correct}", "YELLOW"),
			)
		):
			sys.exit(1)


def ask(question: str) -> bool:
	"""Ask the user {question}.

	resp = input(f'{question}? [Y/n]

	Y returns True
	N returns False
	"""
	with contextlib.suppress(AttributeError):
		if arguments.assume_yes:
			return True
		if arguments.assume_no:
			return False

	resp = input(f"{question} [{YES[0]}/{NO[1]}] ").strip()
	return len(resp) == 0 or resp[0] in (YES[0], YES[1], "Y", "y")


def unauth_ask(question: str) -> bool:
	"""Ask the user if they'd like to accept unauthenticated packages."""
	if not arguments.config.apt.find_b("APT::Get::AllowUnauthenticated", False):
		# If a user is piping something into Nala to bypass this prompt, error because this is unsafe.
		# The option should be passed on the command line so it's explicit what is happening in scripts.
		if arguments.assume_yes or not sys.stdin.isatty():
			sys.exit(
				_(
					"{error} Some packages are unable to be authenticated. Use "
					"'-o APT::Get::AllowUnauthenticated=true' with `--yes`"
				).format(error=ERROR_PREFIX)
			)
		return ask(f"{NOTICE_PREFIX} {question}")
	return True


def compile_regex(regex: str) -> Pattern[str]:
	"""Compile regex and exit on failure."""
	try:
		return re.compile(regex, re.IGNORECASE)
	except re.error as error:
		sys.exit(
			_(
				"{error} failed regex compilation '{error_msg} at position {position}'"
			).format(error=ERROR_PREFIX, error_msg=error.msg, position=error.pos)
		)


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
	# Make sure our directories exist
	NALA_DIR.mkdir(exist_ok=True)
	NALA_LOGDIR.mkdir(exist_ok=True)

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
	return f"{datetime.now().astimezone().strftime('%Y-%m-%d %H:%M:%S %Z')}"


def unit_str(val: int) -> str:
	"""Check integer and figure out what format it should be.

	`unit_str` will return a string with a leading space like " 12 MB".

	You need to strip `unit_str` if you do not want the space.
	"""
	if arguments.config.get_bool("filesize_binary", False):
		base = 1024
		size = ("Bytes", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB")
	else:
		base = 1000
		size = ("Bytes", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB")

	if val > base**3:
		return f"{val/base**3 :.1f} {size[3]}"
	if val > base**2:
		return f"{val/base**2 :.1f} {size[2]}"
	if val > base:
		return f"{round(val/1000) :.0f} {size[1]}"
	return f"{val :.0f} {size[0]}"


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


def get_pkg_version(
	pkg: Package, cand_first: bool = False, inst_first: bool = False
) -> Version:
	"""Get the version."""
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
	if arguments.verbose or arguments.debug:
		print(msg)
	if arguments.debug:
		dprint(from_ansi(f"{msg}").plain, from_verbose=True)
	sys.__stdout__.flush()


def dprint(msg: object, from_verbose: bool = False) -> None:
	"""Print message if debugging, write to log if root.

	from_verbose as true will stop this from printing.

	vprint sends it's messages here to be put in the log.
	"""
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
