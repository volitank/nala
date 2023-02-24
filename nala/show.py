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
"""Functions related to the `show` command."""
from __future__ import annotations

import re
from pathlib import Path
from random import shuffle
from typing import List, cast

from apt.package import BaseDependency, Dependency, Package, Version

from nala import _, color
from nala.cache import Cache
from nala.constants import ERROR_PREFIX, NOTICE_PREFIX, PACSTALL_METADATA
from nala.debfile import NalaBaseDep
from nala.options import arguments
from nala.rich import ascii_replace
from nala.utils import term, unit_str

# NOTE: This is for `nala show`
# NOTE: header info looks like "Package: steam:i386"
SHOW_INFO = _("{header} {info}") + "\n"
URL_PATTERN = re.compile(r"(https?://.*?/.*?/)")


def show_main(num: int, pkg: Package) -> int:
	"""Orchestrate show command with support for all_versions."""
	if num:
		print(f"\n{'='*term.columns}\n")
	count = len(pkg.versions)
	versions = pkg.versions if arguments.all_versions else [pkg.candidate]
	for ver_num, ver in enumerate(versions):
		if ver is None:
			print(
				_("{pkg_name} has no candidate").format(
					pkg_name=color(pkg.name, "YELLOW")
				)
			)
			continue
		if ver_num and not num:
			print(f"\n{'='*term.columns}\n")
		count -= 1
		show_pkg(ver)
	return count


def show_pkg(candidate: Version) -> None:
	"""Start show functions."""
	pkg = candidate.package
	msg = f"{show_format(pkg, candidate)}\n{show_related(candidate)}"
	if candidate.homepage:
		msg += SHOW_INFO.format(header=color(_("Homepage:")), info=candidate.homepage)
	if candidate.size:
		msg += SHOW_INFO.format(
			header=color(_("Download-Size:")), info=unit_str(candidate.size)
		)
	msg += SHOW_INFO.format(
		header=color(_("APT-Sources:")), info=format_sources(candidate, pkg)
	)
	if candidate._translated_records:
		msg += SHOW_INFO.format(
			header=color(_("Description:")),
			info=ascii_replace(candidate._translated_records.long_desc),
		)
	print(msg.strip())


def show_related(candidate: Version) -> str:
	"""Show relational packages."""
	msg = ""
	if candidate.provides:
		msg += SHOW_INFO.format(
			header=color(_("Provides:")),
			info=show_dep([color(name, "GREEN") for name in candidate.provides]),
		)

	if candidate.enhances:
		msg += SHOW_INFO.format(
			header=color(_("Enhances:")),
			info=show_dep([color(pkg[0].name, "GREEN") for pkg in candidate.enhances]),
		)

	if candidate.dependencies:
		depends, pre_depends = split_deps(candidate.dependencies)
		if pre_depends:
			msg += SHOW_INFO.format(
				header=color(_("Pre-Depends:")), info=show_dep(pre_depends)
			)
		if depends:
			msg += SHOW_INFO.format(header=color(_("Depends:")), info=show_dep(depends))

	if candidate.recommends:
		msg += SHOW_INFO.format(
			header=color(_("Recommends:")), info=show_dep(candidate.recommends)
		)

	if candidate.suggests:
		msg += SHOW_INFO.format(
			header=color(_("Suggests:")), info=show_dep(candidate.suggests)
		)

	return msg + additional_related(candidate)


def additional_related(candidate: Version) -> str:
	"""Show breaks, conflicts, replaces."""
	msg = ""
	if replaces := candidate.get_dependencies("Replaces"):
		msg += SHOW_INFO.format(
			header=color(_("Replaces:")),
			info=show_dep([color(pkg[0].name, "GREEN") for pkg in replaces]),
		)
	if conflicts := candidate.get_dependencies("Conflicts"):
		msg += SHOW_INFO.format(header=color(_("Conflicts:")), info=show_dep(conflicts))
	if breaks := candidate.get_dependencies("Breaks"):
		msg += SHOW_INFO.format(header=color(_("Breaks:")), info=show_dep(breaks))
	return msg


def show_format(pkg: Package, candidate: Version) -> str:
	"""Format main section for show command."""
	installed = _("yes") if candidate.is_installed else _("no")
	essential = _("yes") if pkg.essential else _("no")
	maintainer = format_maintainer(f"{candidate.record.get('Maintainer')}".split())

	show = (
		SHOW_INFO.format(header=color(_("Package:")), info=color(pkg.name, "GREEN"))
		+ SHOW_INFO.format(
			header=color(_("Version:")), info=color(candidate.version, "BLUE")
		)
		+ SHOW_INFO.format(
			header=color(_("Architecture:")), info=candidate.architecture
		)
		+ SHOW_INFO.format(header=color(_("Installed:")), info=installed)
		+ SHOW_INFO.format(header=color(_("Priority:")), info=candidate.priority)
		+ SHOW_INFO.format(header=color(_("Essential:")), info=essential)
		+ SHOW_INFO.format(header=color(_("Section:")), info=candidate.section)
		+ SHOW_INFO.format(header=color(_("Source:")), info=candidate.source_name)
	)
	if origin := candidate.origins[0].origin:
		show += SHOW_INFO.format(header=color(_("Origin:")), info=origin)
	show += SHOW_INFO.format(header=color(_("Maintainer:")), info=maintainer)

	if original_maintainer := candidate.record.get("Original-Maintainer"):
		show += SHOW_INFO.format(
			header=color(_("Original-Maintainer:")),
			info=format_maintainer(f"{original_maintainer}".split()),
		)
	if bugs := candidate.record.get("Bugs"):
		show += SHOW_INFO.format(header=color(_("Bugs:")), info=bugs)
	if installed_size := candidate.installed_size:
		show += SHOW_INFO.format(
			header=color(_("Installed-Size:")), info=unit_str(installed_size)
		)
	return show.strip()


def format_maintainer(maintainer: list[str]) -> str:
	"""Format email in maintainer line."""
	maint_list = []
	for line in maintainer:
		if ">" in line:
			line = color(line[1:-1], "BLUE")
			line = color("<") + line + color(">")
		maint_list.append(line)
	return " ".join(maint_list)


def show_dep(dependency: list[Dependency] | list[str]) -> str:
	"""Print dependencies for show."""
	if isinstance(dependency[0], str):
		dependency.sort()
		if len(dependency) > 4:
			return "\n  " + "\n  ".join(cast(list[str], dependency))
		return ", ".join(cast(List[str], dependency))

	dependency = dedupe_deps(cast(List[Dependency], dependency))
	join_list = []
	msg = ""
	same_line = True
	if len(dependency) > 4:
		same_line = False
		msg += "\n"

	for dep_list in dependency:
		dep_print = ""
		for num, dep in enumerate(dep_list):
			assert isinstance(dep, BaseDependency)
			if num == 0:
				dep_print = format_dep(dep, num)
			else:
				dep_print += format_dep(dep, num)

		if same_line:
			join_list.append(dep_print.strip())
			continue
		msg += f"{dep_print}\n"

	return ", ".join(join_list) if same_line else msg.rstrip()


def format_dep(dep: BaseDependency | NalaBaseDep, iteration: int = 0) -> str:
	"""Format dependencies for show."""
	open_paren = color("(")
	close_paren = color(")")
	name = color(dep.name, "GREEN")
	if dep.rawtype in ("Breaks", "Conflicts"):
		name = color(dep.name, "RED")
	relation = color(dep.relation)
	version = color(dep.version, "BLUE")
	indent = color(" | ") if iteration > 0 else "  "

	final = f"{name} {open_paren}{relation} {version}{close_paren}"

	return indent + final if dep.relation else indent + name


def format_sources(candidate: Version, pkg: Package) -> str:
	"""Show apt sources."""
	origin = candidate.origins[0]
	if origin.archive == "now":
		return f"{get_local_source(pkg.shortname)}"
	return (
		f"{source_url(candidate.uris)} {origin.archive}/"
		f"{origin.component} {candidate.architecture} Packages"
	)


def source_url(uris: list[str]) -> str:
	"""Return the source url."""
	shuffle(uris)
	for mirror in uris:
		if regex := re.search(URL_PATTERN, mirror):
			return regex.group()
	return ""


def get_local_source(pkg_name: str) -> str:
	"""Determine the local source and return it."""
	postfixes = ("", "-deb", "-git", "-bin", "-app")
	for postfix in postfixes:
		metadata = PACSTALL_METADATA / (pkg_name + postfix)
		if metadata.exists():
			return parse_pacstall(metadata)
	return _("local install")


def parse_pacstall(pacdata: Path) -> str:
	"""Parse pacstall metadata file."""
	remote = "_remoterepo="
	# _remoterepo="https://github.com/pacstall/pacstall-programs"
	for line in pacdata.read_text().splitlines():
		if line.startswith(remote):
			index = line.index("=") + 1
			return color(line[index:].strip('"'), "BLUE")
	return color("https://github.com/pacstall/pacstall-programs", "BLUE")


def split_deps(
	depend_list: list[Dependency],
) -> tuple[list[Dependency], list[Dependency]]:
	"""Split dependencies into pre-depends and depends."""
	depends: list[Dependency] = []
	pre_depends: list[Dependency] = []
	for depend in depend_list:
		if depend[0].pre_depend:
			pre_depends.append(depend)
			continue
		depends.append(depend)
	return depends, pre_depends


def dedupe_deps(duplicates: list[Dependency]) -> list[Dependency]:
	"""Remove duplicate entries from a list while maintaining the order."""
	deduped = []
	dep_names = []
	for dep in duplicates:
		if dep.rawstr not in dep_names:
			dep_names.append(dep.rawstr)
			deduped.append(dep)
	return deduped


def additional_notice(additional_records: int) -> None:
	"""Print notice of additional records."""
	print(
		_(
			"{notice} There are {num} additional records. Please use the {switch} switch to see them."
		).format(
			notice=NOTICE_PREFIX,
			num=color(additional_records, "YELLOW"),
			switch=color("'-a'", "YELLOW"),
		)
	)


def pkg_not_found(pkg_name: str, cache: Cache, not_found: list[str]) -> None:
	"""Add not found errors to the list."""
	if cache.is_virtual_package(pkg_name):
		return
	if cache.is_secret_virtual(pkg_name):
		not_found.append(
			_("{error} {package} has no version to show").format(
				error=ERROR_PREFIX, package=pkg_name
			)
		)
		return
	not_found.append(
		_("{error} {package} not found").format(error=ERROR_PREFIX, package=pkg_name)
	)
