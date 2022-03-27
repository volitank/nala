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
from pathlib import Path
from typing import Generator, NoReturn, Union, cast

import apt_pkg
from apt.cache import FetchFailedException, LockFailedException
from apt.package import BaseDependency, Dependency, Package, Version

from nala import _, color, color_version
from nala.cache import Cache
from nala.constants import ERROR_PREFIX, NOTICE_PREFIX, WARNING_PREFIX
from nala.debfile import NalaBaseDep, NalaDebPackage, NalaDep
from nala.dpkg import dpkg_error, update_error
from nala.rich import Columns, Text, Tree, from_ansi
from nala.show import SHOW_INFO, format_dep, show_dep
from nala.utils import dprint, eprint, get_installed_dep_names, print_rdeps, term

DEPENDS = color(_("Depends:"))
"""'Depends:'"""
OR_DEPENDS = color(_("Either:"))
"""'Either:'"""
BREAKS = color(_("Breaks:"))
"""'Breaks:'"""
CONFLICTS = color(_("Conflicts:"))
"""'Conflicts:'"""
SECRET_VIRTUAL = _("{pkg_name} is only referenced by name, no packages provides it")
"""'{pkg_name} is a secret virtual package, nothing provides it'"""
BREAKS_MSG = _("{dependency} will break {pkg_name} {version}")
"""'{dependency} will break {pkg_name} {version}'"""
CONFLICTS_MSG = _("{dependency} conflicts with {pkg_name} {version}")
"""'{dependency} conflicts with {pkg_name} {version}'"""

INSTALL_FAILED = _(
	"{error} Installation has failed.\n"
	"If you'd like to file a bug report please include '/var/log/nala/dpkg-debug.log'"
)
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


def apt_error(apt_err: AptErrorTypes) -> NoReturn | None:
	"""Take an error message from python-apt and formats it."""
	msg = str(apt_err)
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
				return None
			sys.exit(1)
		# Sometimes python apt gives us literally nothing to work with.
		# Probably an issue with sources.list. Needs further testing.
		sys.exit(NO_PROPER_ERR.format(error=ERROR_PREFIX, apt_err=repr(apt_err)))

	if "installArchives() failed" in msg:
		eprint(INSTALL_FAILED.format(error=ERROR_PREFIX))
		sys.exit(1)

	if "," in msg:
		err_list = set(msg.split(","))
		for err in err_list:
			if "E:" in err:
				err = err.replace("E:", "")
				eprint(f"{ERROR_PREFIX} {err.strip()}")
				continue
			if "W:" in err:
				err = err.replace("W:", "")
				warn = color(_("Warning:"), "YELLOW")
				eprint(f"{warn} {err.strip()}")
				continue
		sys.exit(1)
	msg = msg.replace("E:", "")
	eprint(f"{ERROR_PREFIX} {msg}")
	if not term.is_su():
		sys.exit(_("Are you root?"))
	sys.exit(1)


def essential_error(pkg_list: list[Text]) -> NoReturn:
	"""Print error message for essential packages and exit."""
	print("=" * term.columns)
	print(
		_("The Following Packages are {essential}").format(
			essential=color("Essential!", "RED")
		)
	)
	print("=" * term.columns)
	term.console.print(Columns(pkg_list, padding=(0, 2), equal=True))
	print("=" * term.columns)
	eprint(
		_("{error} You have attempted to remove {essential}").format(
			error=ERROR_PREFIX, essential=color("essential packages", "RED")
		)
	)
	eprint(
		_("{error} Please use {switch} if you are sure you want to.").format(
			error=ERROR_PREFIX, switch=color("--remove-essential", "YELLOW")
		)
	)
	sys.exit(1)


def pkg_error(
	pkg_list: list[str], cache: Cache, terminate: bool = False, remove: bool = False
) -> None:
	"""Print error for package in list."""
	header = NOTICE_PREFIX if remove else ERROR_PREFIX
	for pkg_name in pkg_list:
		if cache.is_any_virtual(pkg_name):
			if remove:
				eprint(
					_(
						"{error} Virtual Packages like {pkg_name} can't be removed."
					).format(error=header, pkg_name=color(pkg_name, "YELLOW"))
				)
				continue
			eprint(
				_("{error} {pkg_name} has no installation candidate.").format(
					error=header, pkg_name=color(pkg_name, "YELLOW")
				)
			)
			continue
		eprint(
			_("{error} {pkg_name} not found").format(
				error=header, pkg_name=color(pkg_name, "YELLOW")
			)
		)
	if terminate:
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
	msg = str(error)
	if "Invalid archive signature" in msg:
		eprint(
			_("{error} {apt}\n  Unsupported File: {filename}").format(
				error=ERROR_PREFIX,
				apt=msg.replace("E:", ""),
				filename=Path(name).resolve(),
			)
		)
		sys.exit(1)
	eprint(
		_("{error} {apt}\n  Could not read meta data from {filename}").format(
			error=ERROR_PREFIX, apt=msg, filename=Path(name).resolve()
		)
	)
	sys.exit(1)


def format_broken(
	dep: BaseDependency | NalaBaseDep, cache: Cache, arch: str = ""
) -> str:
	"""Format broken dependencies into a Tree, if any."""
	formatted_dep = format_dep(dep, 0).strip()
	dep_name = dep.name
	if arch and ":any" not in dep_name:
		dep_name = f"{dep_name}:{arch}"
		formatted_dep = formatted_dep.replace(dep.name, dep_name)
	# We print nothing on a virtual package
	if cache.is_virtual_package(dep_name):
		return ""
	if cache.is_secret_virtual(dep_name):
		return SECRET_VIRTUAL.format(pkg_name=formatted_dep)
	if dep_name not in cache:
		return _("{pkg_name} but it isn't in the cache").format(pkg_name=formatted_dep)
	if dep.version and not dep.target_versions and not dep.installed_target_versions:
		dep_pkg = cache[dep.name]
		if dep_pkg.candidate and not apt_pkg.check_dep(
			dep_pkg.candidate.version, dep.relation_deb, dep.version
		):
			return _("{pkg_name} but the cache version is {version}").format(
				pkg_name=formatted_dep, version=color_version(dep_pkg.candidate.version)
			)
		# If none of our conditions are met we just fall back to a general error
		return _("{pkg_name} but it cannont be installed").format(
			pkg_name=formatted_dep
		)
	return ""


def format_broken_conflict(
	breaks: list[Dependency] | list[NalaDep], tree_name: str, arch: str = ""
) -> Tree:
	"""Format broken conflict/breaks dependency into a Tree."""
	break_tree = Tree(from_ansi(tree_name))
	for dep in breaks:
		if dep[0].installed_target_versions:
			formatted_dep = format_dep(dep[0], 0).strip()
			break_pkg = dep[0].installed_target_versions[0]
			break_name = break_pkg.package.name
			if arch:
				formatted_dep = formatted_dep.replace(
					dep[0].name, f"{dep[0].name}:{arch}"
				)
				break_name = f"{break_name}:{break_pkg.architecture}"
			break_pkg = dep[0].installed_target_versions[0]
			msg = BREAKS_MSG if tree_name == BREAKS else CONFLICTS_MSG
			break_tree.add(
				from_ansi(
					msg.format(
						dependency=formatted_dep,
						pkg_name=color(break_name, "GREEN"),
						version=color_version(break_pkg.version),
					)
				)
			)
	return break_tree


def breaks_conflicts(
	pkg_name: str, version: Version | NalaDebPackage, arch: str
) -> Generator[Tree, None, None]:
	"""Generate tree objects for breaks and conflict type deps."""
	for dep_type in ("Breaks", "Conflicts"):
		if deps := version.get_dependencies(dep_type):
			dprint(f"{pkg_name} {dep_type}:\n{deps}")
			if (formatted_dep := format_broken_conflict(deps, BREAKS, arch)).children:
				yield formatted_dep


def broken_pkg(  # pylint: disable=too-many-branches
	pkg: Package | NalaDebPackage, cache: Cache
) -> int:
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

	tree = Tree(from_ansi(color(pkg.name, "GREEN")))
	dep_tree = Tree(from_ansi(DEPENDS))
	arch = ""
	if pkg.name.count(":") != 0 and ":all" not in pkg.name and ":any" not in pkg.name:
		arch = pkg.name.split(":")[1]
	dprint(
		f"{pkg.name} Dependencies:\n"
		+ "\n".join(dep.rawstr for dep in version.dependencies)
	)
	for dep in version.dependencies:
		if len(dep) > 1:
			count = 0
			or_tree = Tree(from_ansi(OR_DEPENDS))
			for base_dep in dep:
				if formatted := format_broken(base_dep, cache, arch):
					count += 1
					or_tree.add(from_ansi(formatted))
			if count == len(dep):
				tree.add(or_tree)
		elif formatted := format_broken(dep[0], cache, arch):
			dep_tree.add(from_ansi(formatted))

	if dep_tree.children:
		tree.add(dep_tree)

	for formatted_break in breaks_conflicts(pkg.name, version, arch):
		tree.add(formatted_break)

	if tree.children:
		term.console.print(tree, soft_wrap=True)
		ret_count += 1
		print()
	return ret_count


def broken_error(
	broken_list: list[Package] | list[NalaDebPackage],
	cache: Cache,
	installed_pkgs: bool | tuple[Package, ...] = False,
) -> int | NoReturn:
	"""Handle printing of errors due to broken packages."""
	if isinstance(installed_pkgs, tuple):
		total_deps = get_installed_dep_names(installed_pkgs)
	# We have to clear the changes from the cache
	# before we can calculate why the packages are broken.
	cache.clear()
	ret_count = 0
	for pkg in broken_list:
		# if installed_pkgs exist then we are removing.
		if installed_pkgs and pkg.name in total_deps:
			ret_count += 1
			print_rdeps(pkg.name, cast(tuple[Package], installed_pkgs))
			continue
		ret_count += broken_pkg(pkg, cache)
	if not ret_count:
		return ret_count
	eprint(
		_("{notice} The information above may be able to help").format(
			notice=NOTICE_PREFIX
		)
	)
	sys.exit(_("{error} You have held broken packages").format(error=ERROR_PREFIX))


def unmarked_error(pkgs: list[Package]) -> None:
	"""Print error messages related to the fixer unmarking packages requested for install."""
	terminate = False
	for pkg in pkgs:
		if not pkg.marked_upgrade or pkg.marked_install:
			terminate = True
			print(
				_("{package} has been unmarked.").format(
					package=color(pkg.name, "GREEN"),
				)
			)
	if terminate:
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


def print_broken(pkg_name: str, candidate: Version) -> None:
	"""Print broken packages information."""
	print("=" * term.columns)
	version = color("(") + color(candidate.version, "BLUE") + color(")")
	print(f"{color('Package:', 'YELLOW')} {color(pkg_name, 'GREEN')} {version}")
	msg = ""
	if conflicts := candidate.get_dependencies("Conflicts"):
		msg += SHOW_INFO.format(
			header=color(_("Conflicts:"), "YELLOW"), info=show_dep(conflicts)
		)
	if breaks := candidate.get_dependencies("Breaks"):
		msg += SHOW_INFO.format(
			header=color(_("Breaks:"), "YELLOW"), info=show_dep(breaks)
		)
	if candidate.dependencies:
		msg += SHOW_INFO.format(
			header=color(_("Depends:"), "YELLOW"), info=show_dep(candidate.dependencies)
		)
	print(msg)
