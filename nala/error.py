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
"""Functions for Nala errors."""
from __future__ import annotations

import sys
from pathlib import Path
from typing import Iterable, NoReturn, Union

import apt_pkg
from apt.cache import FetchFailedException, LockFailedException
from apt.package import BaseDependency, Dependency, Package, Version

from nala import ROOT, _, color, color_version
from nala.cache import Cache
from nala.constants import ERROR_PREFIX, NOTICE_PREFIX, WARNING_PREFIX
from nala.debfile import NalaBaseDep, NalaDebPackage, NalaDep
from nala.dpkg import dpkg_error, update_error
from nala.rich import Columns, Text
from nala.search import BOT_LINE, LINE, TOP_LINE
from nala.show import format_dep
from nala.utils import dprint, eprint, term

DEPENDS = color(_("Depends:"))
"""'Depends:'"""
OR_DEPENDS = color(_("Either:"))
"""'Either:'"""
BREAKS = color(_("Breaks:"))
"""'Breaks:'"""
CONFLICTS = color(_("Conflicts:"))
"""'Conflicts:'"""
SECRET_VIRTUAL = _("{package} is only referenced by name, no packages provides it")
"""'{package} is a secret virtual package, nothing provides it'"""
BREAKS_MSG = _("{dependency} will break {package} {version}")
"""'{dependency} will break {package} {version}'"""
CONFLICTS_MSG = _("{dependency} conflicts with {package} {version}")
"""'{dependency} conflicts with {package} {version}'"""

NO_PROPER_ERR = _(
	"{error} python-apt gave us {apt_err} This isn't a proper error as it's empty"
)
AptErrorTypes = Union[
	FetchFailedException, LockFailedException, apt_pkg.Error, SystemError
]


class ExitCode:  # pylint: disable=too-few-public-methods
	"""Constants for Exit Codes."""

	SIGINT = 130
	SIGTERM = 143


class FileDownloadError(Exception):
	"""Exception class for passing errors.

	ERRHASH: 1 'Hash Sum is mismatched.'
	ENOENT: 2 'No such file or directory.'
	ERRSIZE: 3 'Size is mismatched.'
	"""

	ERRHASH = 1
	ENOENT = 2
	ERRSIZE = 3

	def __init__(  # pylint: disable=too-many-arguments
		self,
		error_str: str = "",
		errno: int = 0,
		filename: str = "",
		expected: str = "",
		received: str = "",
	) -> None:
		"""Define error properties."""
		super().__init__(error_str)
		self.error_str = error_str
		self.errno = errno
		self.filename = filename
		self.expected = expected
		self.received = received


class ParserError(Exception):
	"""Exception class for errors with parsing."""


# Should probably refactor this in the future. For now just disable the warning.
# pylint: disable=too-many-branches
def apt_error(apt_err: AptErrorTypes, update: bool = False) -> NoReturn | None:
	"""Take an error message from python-apt and formats it."""
	msg = f"{apt_err}"
	if not msg:
		if update_error:
			print()
			bad_mirror = False
			for line in update_error:
				bad_mirror = "Connection failed" in line
				eprint(line)
			if bad_mirror:
				eprint(
					_(
						"{notice} Some index files failed to download. "
						"They have been ignored, or old ones used instead."
					).format(notice=NOTICE_PREFIX)
				)

			if update:
				sys.exit(1)
		# Sometimes python apt gives us literally nothing to work with.
		# Probably an issue with sources.list. Needs further testing.
		if update:
			sys.exit(NO_PROPER_ERR.format(error=ERROR_PREFIX, apt_err=repr(apt_err)))
		return None

	if "installArchives() failed" in msg:
		eprint(_("{error} Installation has failed.").format(error=ERROR_PREFIX))
		eprint(
			_(
				"If you'd like to file a bug report please include '{debug_file}'"
			).format(debug_file=f"{ROOT}/var/log/nala/dpkg-debug.log")
		)
		sys.exit(1)

	if "," in msg:
		err_list = set(msg.split(","))
		for err in err_list:
			if "E:" in err:
				eprint(f"{ERROR_PREFIX} {err.replace('E:', '').strip()}")
				continue
			if "W:" in err:
				eprint(f"{WARNING_PREFIX} {err.replace('W:', '').strip()}")
				continue
		if update:
			sys.exit(1)
		return None

	eprint(f"{ERROR_PREFIX} {msg.replace('E:', '').strip()}")
	if not term.is_su():
		sys.exit(_("Are you root?"))
	if update or "Failed to lock directory" in msg:
		sys.exit(1)
	return None


def essential_error(pkg_list: list[Text]) -> NoReturn:
	"""Print error message for essential packages and exit."""
	print("=" * term.columns)
	print(_("{error} The following packages are essential!").format(error=ERROR_PREFIX))
	print("=" * term.columns)
	term.console.print(Columns(pkg_list, padding=(0, 2), equal=True))
	print("=" * term.columns)
	eprint(
		_("{error} You have attempted to remove essential packages").format(
			error=ERROR_PREFIX
		)
	)
	eprint(
		_("{error} Please use {switch} if you are sure you want to.").format(
			error=ERROR_PREFIX, switch=color("--remove-essential", "YELLOW")
		)
	)
	sys.exit(1)


def pkg_error(pkg_list: list[str], cache: Cache) -> NoReturn:
	"""Print error for package in list."""
	for pkg_name in pkg_list:
		if cache.is_any_virtual(pkg_name):
			eprint(
				_("{error} {package} has no installation candidate.").format(
					error=ERROR_PREFIX, package=color(pkg_name, "YELLOW")
				)
			)
			continue
		eprint(
			_("{error} {package} not found").format(
				error=ERROR_PREFIX, package=color(pkg_name, "YELLOW")
			)
		)
	sys.exit(1)


def print_dpkg_errors() -> None:
	"""Format and print dpkg errors if there are any."""
	if not dpkg_error:
		return
	# for line in dedupe_list(dpkg_error):
	for line in dpkg_error:
		if "dpkg:" in line:
			line = line.replace("dpkg:", "")
			if "warning:" in line:
				line = line.replace("warning:", "")
				if "downgrading" in line:
					line = line.replace("downgrading", "Downgraded")
				eprint(f"\n{WARNING_PREFIX} {line.strip()}")
				continue
			eprint(f"\n{ERROR_PREFIX} {line.strip()}")
			continue
		if "Errors were encountered" in line or "Processing was halted" in line:
			eprint(f"\n{line}")
			continue
		eprint(line)


def local_deb_error(error: apt_pkg.Error, name: str) -> NoReturn:
	"""Print what is wrong with the .deb and exit."""
	msg = f"{error}"
	if "Invalid archive signature" in msg:
		eprint(
			_("{error} {apt_error}\n  Unsupported File: {filename}").format(
				error=ERROR_PREFIX,
				apt_error=msg.replace("E:", "").strip(),
				filename=Path(name).resolve(),
			)
		)
		sys.exit(1)
	eprint(
		_("{error} {apt_error}\n  Could not read meta data from {filename}").format(
			error=ERROR_PREFIX, apt_error=msg, filename=Path(name).resolve()
		)
	)
	sys.exit(1)


class BrokenError:
	"""Calculate and print broken dependencies."""

	def __init__(
		self,
		cache: Cache,
		broken_list: Iterable[Package] | list[NalaDebPackage] | None = None,
	) -> None:
		"""Calculate and print broken install dependencies."""
		self.cache = cache
		self.broken_list = broken_list
		self.provides = (
			{
				ppkg
				for pkg in broken_list
				if isinstance(pkg, Package) and pkg.candidate
				for ppkg in pkg.candidate.provides
			}
			if broken_list
			else set()
		)

	def broken_install(self) -> int | NoReturn:
		"""Handle printing of errors due to broken packages."""
		# We have to clear the changes from the cache
		# before we can calculate why the packages are broken.
		if not self.broken_list:
			return 0
		self.cache.clear()
		if ret_count := sum(self.broken_pkg(pkg) for pkg in self.broken_list):
			self._print_held_error()
		else:
			return ret_count

	def broken_remove(self, broken_list: list[Package]) -> NoReturn:
		"""Calculate and print broken remove dependencies."""
		installed = tuple(
			pkg for pkg in self.cache if pkg.installed and pkg.installed.dependencies
		)
		self.cache.clear()
		for pkg in (
			pkg.name
			for pkg in broken_list
			if pkg.name in self._installed_dep_names(installed)
		):
			self._print_rdeps(pkg, installed)
		self._print_held_error()

	def held_pkgs(self, protected: set[Package]) -> None:
		"""Print packages that have been held back."""
		if not self.broken_list:
			# There is really not a chance of this happening
			return
		print(color(_("The following packages were kept back:"), "YELLOW"))
		if undetermined := [
			pkg for pkg in self.broken_list if not self.broken_pkg(pkg)
		]:
			print(
				color(
					_("The following were held due to exclusions:")
					if protected
					else _("Nala was unable to determine why these were held:")
				)
			)
			print(f"  {', '.join(color(pkg.name, 'YELLOW') for pkg in undetermined)}")

	def broken_pkg(self, pkg: Package | NalaDebPackage) -> int:
		"""Calculate and print broken Dependencies."""
		ret_count = 0
		version: NalaDebPackage | Version | None
		if isinstance(pkg, NalaDebPackage):
			version = pkg
		elif not (version := pkg.candidate):
			# We do this in case a broken package is locally installed.
			if not pkg.installed:
				return ret_count
			version = pkg.installed

		tree: list[str] = []
		dep_tree: list[str] = []
		arch = self._arch(pkg.name)

		indent = "    "
		if formatted_break := self.breaks_conflicts(pkg.name, version, arch):
			indent = LINE

		for dep in version.dependencies:
			dep_tree.extend(self._dep_tree(dep, arch, indent))

		if dep_tree:
			dep_tree.insert(0, f"{TOP_LINE if formatted_break else BOT_LINE} {DEPENDS}")
			dep_tree.append(dep_tree.pop().replace(TOP_LINE, BOT_LINE))
			tree.extend(dep_tree)

		tree.extend(f"{TOP_LINE} {_break}" for _break in formatted_break)
		if tree:
			tree.append(tree.pop().replace(TOP_LINE, BOT_LINE))
			tree.insert(0, color(pkg.name, "GREEN"))
			print("\n".join(tree), end="\n\n")
			ret_count += 1
		return ret_count

	def _dep_tree(
		self,
		dep: NalaDep | Dependency,
		arch: str,
		indent: str,
	) -> list[str]:
		dep_tree: list[str] = []
		if len(dep) > 1:
			count = 0
			or_tree = []
			for base_dep in dep:
				if formatted := self.format_broken(base_dep, arch):
					count += 1
					or_tree.append(formatted)
			if count == len(dep):
				start = f"{indent}{color(TOP_LINE, 'MAGENTA')}"
				for num, msg in enumerate(or_tree):
					if not num:
						dep_tree.append(f"{start} {msg}")
						continue
					dep_tree.append(f"{start} or {msg}")

		elif formatted := self.format_broken(dep[0], arch):
			dep_tree.append(f"{indent}{TOP_LINE} {formatted}")
		return dep_tree

	def format_broken(self, dep: BaseDependency | NalaBaseDep, arch: str = "") -> str:
		"""Format broken dependencies into a Tree, if any."""
		formatted_dep = format_dep(dep, 0).strip()
		dep_name = dep.name
		if arch and ":any" not in dep_name:
			dep_name = f"{dep_name}:{arch}"
			formatted_dep = formatted_dep.replace(dep.name, dep_name)
		# We print nothing on a virtual package
		if self.cache.is_virtual_package(dep_name):
			return ""
		if self.cache.is_secret_virtual(dep_name):
			return SECRET_VIRTUAL.format(package=formatted_dep)
		if dep_name not in self.cache:
			return _("{package} but it isn't in the cache").format(
				package=formatted_dep
			)
		# This means that the dependency doesn't have the right version in the cache
		if (
			dep.version
			and not dep.target_versions
			and not dep.installed_target_versions
		):
			dep_pkg = self.cache[dep.name]
			if (candidate := dep_pkg.candidate) and not apt_pkg.check_dep(
				candidate.version, dep.relation_deb, dep.version
			):
				return _("{package} but the cache version is {version}").format(
					package=formatted_dep,
					version=color_version(candidate.version),
				)
			# If none of our conditions are met we just fall back to a general error
			return _("{package} but it cannot be installed").format(
				package=formatted_dep
			)
		return ""

	def breaks_conflicts(
		self,
		pkg_name: str,
		version: Version | NalaDebPackage,
		arch: str,
	) -> tuple[str, ...]:
		"""Generate tree objects for breaks and conflict type deps."""
		break_conflict: list[str] = []
		for dep_type in ("Breaks", "Conflicts"):
			if deps := version.get_dependencies(dep_type):
				dprint(f"{pkg_name} {dep_type}:\n{deps}")
				break_conflict.extend(
					self.format_broken_conflict(
						deps,
						BREAKS_MSG if dep_type == "Breaks" else CONFLICTS_MSG,
						arch,
					)
				)
		return tuple(break_conflict)

	def format_broken_conflict(
		self,
		breaks: list[Dependency] | list[NalaDep],
		dep_string: str,
		arch: str = "",
	) -> list[str]:
		"""Format broken conflict/breaks dependency into a Tree."""
		break_tree: list[str] = []
		for dep in breaks:
			if not (target_versions := dep.target_versions):
				continue
			if installed_versions := tuple(
				ver for ver in target_versions if ver.is_installed
			):
				break_tree.append(
					dep_string.format(
						dependency=self._dependency_name(dep, arch),
						package=color(
							self._break_pkg_name(installed_versions[0], arch), "GREEN"
						),
						version=color_version(installed_versions[0].version),
					)
				)

			if not (target_provides := target_versions[0].provides):
				continue

			if conflict := self.format_conflict(dep, target_provides, dep_string):
				break_tree.append(conflict)
		return break_tree

	def format_conflict(
		self,
		dep: Dependency | NalaDep,
		target_provides: list[str],
		dep_string: str,
	) -> str | None:
		"""Format a conflicting package that isn't installed."""
		return (
			next(
				(
					dep_string.format(
						dependency=format_dep(dep[0]).strip(),
						package=color(provide_name, "GREEN"),
						version="",
					)
					for provide_name in target_provides
					if dep[0].name != provide_name and provide_name in self.provides
				),
				None,
			)
			if self.provides
			else None
		)

	@staticmethod
	def _break_pkg_name(version: Version, arch: str) -> str:
		"""Get the name of the package that is broken."""
		return (
			f"{version.package.name}:{version.architecture}"
			if arch
			else version.package.name
		)

	@staticmethod
	def _dependency_name(dep: Dependency | NalaDep, arch: str) -> str:
		"""Get the formatted dependency name."""
		return (
			format_dep(dep[0]).replace(dep[0].name, f"{dep[0].name}:{arch}")
			if arch
			else format_dep(dep[0])
		).strip()

	@staticmethod
	def _arch(pkg_name: str) -> str:
		return (
			pkg_name.split(":")[1]
			if ":" in pkg_name
			and all(substring not in pkg_name for substring in (":all", ":any"))
			else ""
		)

	@staticmethod
	def unmarked_error(pkgs: list[Package]) -> None:
		"""Print error messages related to the fixer unmarking packages requested for install."""
		for pkg in pkgs:
			if not pkg.marked_upgrade or pkg.marked_install:
				print(
					_("{package} has been unmarked.").format(
						package=color(pkg.name, "GREEN"),
					)
				)
		print(
			_("Try {switch} if you're sure they can be installed.").format(
				switch=color("--no-fix-broken", "YELLOW")
			)
		)
		sys.exit(
			_("{error} Some packages were unable to be installed.").format(
				error=ERROR_PREFIX
			)
		)

	@staticmethod
	def _installed_dep_names(installed_pkgs: tuple[Package, ...]) -> tuple[str, ...]:
		"""Iterate installed pkgs and return all of their deps in a tuple.

		This is so we can reduce iterations when checking reverse depends.
		"""
		total_deps = set()
		for pkg in installed_pkgs:
			if not (pkg_installed := pkg.installed):
				continue
			for deps in pkg_installed.dependencies:
				for dep in deps:
					total_deps.add(dep.name)
		return tuple(total_deps)

	@staticmethod
	def _print_rdeps(name: str, installed_pkgs: tuple[Package, ...]) -> int:
		"""Print the installed reverse depends of a package."""
		msg = color(
			_("Installed packages that depend on {package}").format(
				package=color(name, "GREEN")
			)
			+ "\n",
			"YELLOW",
		)
		for pkg in installed_pkgs:
			if not (pkg_installed := pkg.installed):
				continue
			for dep in pkg_installed.dependencies:
				if name in dep.rawstr:
					dep_msg = f"  {color(pkg.name, 'GREEN')}"
					if pkg.essential:
						dep_msg = _("{package} is an Essential package!").format(
							package=dep_msg
						)
					msg += f"{dep_msg}\n"
					break
		print(msg.strip())
		return 1

	@staticmethod
	def _print_held_error() -> NoReturn:
		"""Print the held broken error and exit."""
		eprint(
			_("{notice} The information above may be able to help").format(
				notice=NOTICE_PREFIX
			)
		)
		sys.exit(_("{error} You have held broken packages").format(error=ERROR_PREFIX))
