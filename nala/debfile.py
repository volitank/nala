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
"""Module for Nala debfile and dependency subclasses."""

from __future__ import annotations

import contextlib
from typing import List, cast

import apt_pkg
from apt.debfile import DebPackage
from apt.package import Version, VersionList

from nala.cache import Cache


class NalaDebPackage(DebPackage):
	"""A subclass for DebPackage to have attributes similar to Package."""

	def __init__(self, filename: str, cache: Cache) -> None:
		"""Subclass for DebPackage."""
		super().__init__(filename, cache)
		self.filename = filename
		self._cache = cache

	@property
	def name(self) -> str:
		"""Return the package name. This may have :arch appended."""
		return self.pkgname

	def _get_depends(self, _type: str) -> list[list[tuple[str, str, str]]]:
		"""List of packages on which this package depends on."""
		depends = []
		with contextlib.suppress(KeyError):
			depends.extend(apt_pkg.parse_depends(self._sections[_type], False))
		return depends

	def get_dependencies(self, *types: str) -> list[NalaDep]:
		"""Return a list of Dependency objects for the given types.

		Multiple types can be specified. Possible types are:
		'Breaks', 'Conflicts', 'Depends', 'Enhances', 'PreDepends',
		'Recommends', 'Replaces', 'Suggests'

		Additional types might be added in the future.
		"""
		depends_list = []
		for _type in types:
			for dep in self._get_depends(_type):
				base_deps = [
					NalaBaseDep(*dep_or, _type, cast(Cache, self._cache))
					for dep_or in dep
				]
				depends_list.append(NalaDep(base_deps, _type))
		return depends_list

	def installed_size(self) -> int:
		"""Get the installed size. Returns 0 if it cannot be found."""
		with contextlib.suppress(KeyError):
			return int(self._sections["Installed-Size"])
		return 0

	@property
	def dependencies(self) -> list[NalaDep]:
		"""Return the dependencies of the package version."""
		return self.get_dependencies("PreDepends", "Depends")


class NalaBaseDep:
	"""Base Dependency class to contain debfile deps."""

	def __init__(  # pylint: disable=too-many-arguments
		self,
		name: str,
		version: str,
		relation: str,
		rawtype: str,
		cache: Cache,
	) -> None:
		"""Initialize Base Dependency class to contain debfile deps."""
		self.name = name
		self.relation = relation
		self.relation_deb = relation
		self.version = version
		self.rawtype = rawtype
		self.cache = cache

	def __repr__(self) -> str:
		"""Return a string representation of the instance."""
		return (
			f"<NalaBaseDep: name:{self.name} relation:{self.relation}"
			f" version:{self.version} rawtype:{self.rawtype}>"
		)

	@property
	def rawstr(self) -> str:
		"""Return a string representation of the dependency.

		Returns the string representation of the dependency as it would be
		written in the debian/control file. The string representation does not
		include the type of the dependency.

		Example for an unversioned dependency:
		        python3

		Example for a versioned dependency:
		        python3 >= 3.2

		.. versionadded:: 1.0.0
		"""
		if self.version:
			return f"{self.name} {self.relation} {self.version}"
		return self.name

	@property
	def target_versions(self) -> VersionList:
		"""Return the target versions if they exist."""
		try:
			pkg = self.cache[self.name]
		except KeyError:
			return cast(VersionList, [])
		return pkg.versions

	@property
	def installed_target_versions(self) -> list[Version]:
		"""Return the installed target versions if they exist."""
		return [ver for ver in self.target_versions if ver.is_installed]


# Nuitka doesn't seem to like typing with the lower case list on this class.
class NalaDep(List[NalaBaseDep]):
	"""Dependency class to contain debfile deps."""

	def __init__(self, base_deps: list[NalaBaseDep], rawtype: str) -> None:
		"""Dependency class to contain debfile deps."""
		super().__init__(base_deps)
		self.rawtype = rawtype

	def __repr__(self) -> str:
		"""Return a string representation of the instance."""
		return f"<NalaDep: [{', '.join(repr(base_dep) for base_dep in self)}]>"

	@property
	def rawstr(self) -> str:
		"""Return a string representation of the Or-group of dependencies.

		Returns the string representation of the Or-group of dependencies as it
		would be written in the debian/control file.  The string representation
		does not include the type of the Or-group of dependencies.

		Example:
		-------
		        python2 >= 2.7 | python3

		"""
		return " | ".join(bd.rawstr for bd in self)

	@property
	def target_versions(self) -> list[Version]:
		"""List all Version objects which satisfy this Or-group of deps."""
		tvers: set[Version] = set()
		for base_dep in self:
			for tver in base_dep.target_versions:
				tvers.add(tver)
		return list(tvers)

	@property
	def installed_target_versions(self) -> list[Version]:
		"""List all Version installed Version objects which satisfy this dep."""
		return [ver for ver in self.target_versions if ver.is_installed]
