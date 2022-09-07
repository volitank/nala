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
"""The Cache subclass module."""
from __future__ import annotations

import contextlib
import fnmatch
import sys
from typing import TYPE_CHECKING, Generator

import apt_pkg
from apt.cache import Cache as _Cache
from apt.package import Package

from nala import _, color, color_version
from nala.constants import ERROR_PREFIX, NOTICE_PREFIX, WARNING_PREFIX
from nala.options import arguments
from nala.rich import Columns, from_ansi
from nala.utils import dprint, eprint, term

if TYPE_CHECKING:
	from nala.debfile import NalaDebPackage
	from nala.dpkg import InstallProgress, UpdateProgress


PACKAGES_CAN_BE_UPGRADED = "\n" + _("The following {total} packages can be upgraded:")
NOT_CANDIDATE = color("[") + color(_("Not candidate version"), "YELLOW") + color("]")


class Cache(_Cache):
	"""Subclass of apt.cache to add features."""

	def commit_pkgs(
		self,
		install_progress: InstallProgress,
		update_progress: UpdateProgress,
		local_debs: list[NalaDebPackage] | None = None,
	) -> bool:
		"""Apply the marked changes to the cache."""
		if local_debs:
			res = install_archives(
				[pkg.filename for pkg in local_debs if pkg.filename], install_progress
			)
			install_progress.finish_update()
			return res == 0
		with apt_pkg.SystemLock():
			apt = apt_pkg.PackageManager(self._depcache)
			fetcher = apt_pkg.Acquire(update_progress)
			with self._archive_lock:
				while True:
					self._fetch_archives(fetcher, apt)
					res = install_archives(apt, install_progress)
					if res == apt.RESULT_COMPLETED:
						break
					if res == apt.RESULT_FAILED:
						raise SystemError("installArchives() failed")
					if res != apt.RESULT_INCOMPLETE:
						raise SystemError(
							f"internal-error: unknown result code from InstallArchives: {res}"
						)
					# reload the fetcher for media swapping
					fetcher.shutdown()
		return res == apt.RESULT_COMPLETED

	def is_secret_virtual(self, pkg_name: str) -> bool:
		"""Return True if the package is secret virtual.

		Secret virtual packages provide nothing, and have no versions.

		cache.is_virtual_package() returns True only if the virtual package
		has something that it can provide.
		"""
		try:
			pkg = self._cache[pkg_name]
			return not pkg.has_provides and not pkg.has_versions
		except KeyError:
			return False

	def is_any_virtual(self, pkgname: str) -> bool:
		"""Return whether the package is a virtual package.

		This is used if we only care if it's virtual, but not what type.
		"""
		try:
			pkg = self._cache[pkgname]
		except KeyError:
			return False
		else:
			return not pkg.has_versions

	def glob_filter(self, pkg_names: list[str], show: bool = False) -> list[str]:
		"""Filter provided packages and glob *.

		Returns a new list of packages matching the glob.

		If there is nothing to glob it returns the original list.
		"""
		if "*" not in f"{pkg_names}":
			return pkg_names

		new_packages: list[str] = []
		glob_failed = False
		for pkg_name in pkg_names:
			if "*" in pkg_name:
				dprint(f"Globbing: {pkg_name}")
				glob = fnmatch.filter(self.get_pkg_names(show), pkg_name)
				if not glob:
					glob_failed = True
					eprint(
						_(
							"{error} unable to find any packages by globbing {pkg}"
						).format(error=ERROR_PREFIX, pkg=color(pkg_name, "YELLOW"))
					)
					continue
				new_packages.extend(glob)
			else:
				new_packages.append(pkg_name)

		if glob_failed:
			sys.exit(1)
		new_packages.sort()
		dprint(f"List after globbing: {new_packages}")
		return new_packages

	def get_pkg_names(self, show: bool = False) -> Generator[str, None, None]:
		"""Generate all real packages, or packages that can provide something."""
		for pkg in self._cache.packages:  # pylint: disable=not-an-iterable
			pretty_name = pkg.get_fullname(pretty=True)
			if not show and self.is_virtual_package(pretty_name):
				provides = self.get_providing_packages(pretty_name)
				if not provides or len(provides) > 1:
					continue
			# For some reason a virtual package $kernel exists and can't be accessed.
			if pretty_name.startswith("$"):
				continue
			if pkg.has_versions or pkg.has_provides:
				yield pretty_name

	def virtual_filter(self, pkg_names: list[str], remove: bool = False) -> list[str]:
		"""Filter package to check if they're virtual."""
		new_names = set()
		for pkg_name in pkg_names:
			if pkg_name in self:
				new_names.add(pkg_name)
				continue
			if (vpkg := self.check_virtual(pkg_name, remove)) and isinstance(
				vpkg, Package
			):
				new_names.add(vpkg.name)
				continue
			new_names.add(pkg_name)
		dprint(f"Virtual Filter: {new_names}")
		return sorted(new_names)

	def what_replaces(self, pkg_name: str) -> Generator[str, None, None]:
		"""Generate packages that replace the given name."""
		for pkg in self._cache.packages:  # pylint: disable=not-an-iterable
			if cand := self._depcache.get_candidate_ver(pkg):
				with contextlib.suppress(KeyError):
					replaces = cand.depends_list["Replaces"]
					target = replaces[0][0].target_pkg
					if pkg_name == target.name:
						yield pkg.get_fullname(pretty=True)

	def check_virtual(self, pkg_name: str, remove: bool = False) -> Package | bool:
		"""Check if the package is virtual."""
		if not self.is_virtual_package(pkg_name):
			return False
		if len(provides := self.get_providing_packages(pkg_name)) == 1:
			print_selecting_pkg(provides[0].name, pkg_name)
			return self[provides[0]]
		if remove:
			eprint(
				_("{warn} Virtual Packages like {package} can't be removed.").format(
					warn=WARNING_PREFIX, package=color(pkg_name, "YELLOW")
				)
			)
			return True
		if not provides:
			if provides := self.get_providing_packages(pkg_name, candidate_only=False):
				print_virtual_pkg(pkg_name, provides, not_candidate=True)
				return True
		print_virtual_pkg(pkg_name, provides)
		return True

	def purge_removed(self) -> None:
		"""Make sure everything marked as removed is getting purged."""
		if not arguments.is_purge():
			return
		for pkg in self:
			if pkg.marked_delete:
				pkg.mark_delete(purge=True)

	def protect_upgrade_pkgs(self, exclude: list[str] | None) -> set[Package]:
		"""Mark excluded packages as protected."""
		protected: set[Package] = set()
		if not exclude:
			return protected
		resolver = apt_pkg.ProblemResolver(self._depcache)
		for pkg_name in self.glob_filter(exclude):
			if pkg_name in self:
				pkg = self[pkg_name]
				if pkg.is_upgradable:
					print(
						_("Protecting {package} from upgrade").format(
							package=color(pkg_name, "GREEN")
						)
					)
					resolver.protect(self._cache[pkg_name])
					protected.add(pkg)
				elif pkg.is_auto_removable:
					print(
						_("Protecting {package} from auto-removal").format(
							package=color(pkg_name, "GREEN")
						)
					)
					resolver.protect(self._cache[pkg_name])
					protected.add(pkg)
		return protected

	def upgradable_pkgs(self) -> Generator[Package, None, None]:
		"""Generate upgradable packages."""
		return (pkg for pkg in self if pkg.is_upgradable)

	def print_upgradable(self) -> None:
		"""Print packages that are upgradable."""
		if arguments.config.get_bool("update_show_packages"):
			if upgradable := [
				# format will look like "python3-pip (22.1.1+dfsg-1) -> (22.2+dfsg-1)"
				from_ansi(
					f"{color(pkg.name, 'GREEN')} "
					f"{color_version(pkg.installed.version)} -> {color_version(pkg.candidate.version)}"
				)
				for pkg in self.upgradable_pkgs()
				if pkg.installed and pkg.candidate
			]:
				print(PACKAGES_CAN_BE_UPGRADED.format(total=color(len(upgradable))))
				term.console.print(Columns(upgradable, padding=(0, 2), equal=True))
				return

		elif total_pkgs := len(tuple(self.upgradable_pkgs())):
			print(
				_(
					"{total} packages can be upgraded. Run '{command}' to see them."
				).format(
					total=color(total_pkgs, "YELLOW"),
					command=color("nala list --upgradable", "GREEN"),
				)
			)
			return

		print(color(_("All packages are up to date.")))


def install_archives(
	apt: apt_pkg.PackageManager | list[str], install_progress: InstallProgress
) -> int:
	"""Install the archives."""
	install_progress.start_update()

	if did_unlock := apt_pkg.pkgsystem_is_locked():
		apt_pkg.pkgsystem_unlock_inner()

	try:
		res = install_progress.run_install(apt)
	finally:
		if did_unlock:
			apt_pkg.pkgsystem_lock_inner()

	install_progress.finish_update()
	return res


def print_virtual_pkg(
	pkg_name: str, provides: list[Package], not_candidate: bool = False
) -> None:
	"""Print the virtual package string."""
	print(
		_("{package} is a virtual package provided by:").format(
			package=color(pkg_name, "GREEN")
		)
	)
	print(
		"".join(
			[
				f"\n  {color(pkg.name, 'GREEN')} {color_version(pkg.candidate.version)} "
				f"{NOT_CANDIDATE if not_candidate else ''}"
				for pkg in provides
				if pkg.candidate
			]
		).strip("\n")
	)
	print(_("You should select just one."))


def print_selecting_pkg(provider: str, pkg_name: str) -> None:
	"""Print that we are selecting a different package."""
	print(
		_(
			"{notice} Selecting {provider}\n  Instead of virtual package {package}"
		).format(
			notice=NOTICE_PREFIX,
			provider=color(provider, "GREEN"),
			package=color(pkg_name, "GREEN"),
		),
		end="\n\n",
	)
