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
"""Module for printing the transaction summary."""
from __future__ import annotations

from dataclasses import dataclass
from itertools import zip_longest
from typing import Generator, Iterable

from nala import _, color, console
from nala.cache import Cache
from nala.options import arguments
from nala.rich import HORIZONTALS, OVERFLOW, Column, Group, Table, Text, Tree, from_ansi
from nala.utils import NalaPackage, PackageHandler, dprint, term, unit_str

# NOTE: The following are the headers for the transaction summary.
# NOTE: Package:        Version:     Size:
# NOTE: ansible-core    2.12.4-1    1.2 MB
PACKAGE, VERSION, SIZE, OLD_VERSION, NEW_VERSION, EITHER = _(
	"Package/Version/Size/Old Version/New Version/Either"
).split("/")
# NOTE: Verb Tenses are [ "Present/Present Participle/Past" ]
# NOTE: This ends up looking like [ "Auto-Purge 20 Packages" ]
_AUTO_PURGE, _AUTO_PURGING, _AUTO_PURGED = _(
	"Auto-Purge/Auto-Purging/Auto-Purged",
).split("/")
# NOTE: Verb Tenses are [ "Present/Present Participle/Past" ]
# NOTE: This ends up looking like [ "Auto-Remove 20 Packages" ]
_AUTO_REMOVE, _AUTO_REMOVING, _AUTO_REMOVED = _(
	"Auto-Remove/Auto-Removing/Auto-Removed",
).split("/")
# NOTE: Verb Tenses are [ "Present/Present Participle/Past" ]
# NOTE: This ends up looking like [ "Remove 20 Packages" ]
_REMOVE, _REMOVING, _REMOVED = _(
	"Remove/Removing/Removed",
).split("/")
# NOTE: Verb Tenses are [ "Present/Present Participle/Past" ]
# NOTE: This ends up looking like [ "Purge 20 Packages" ]
_PURGE, _PURGING, _PURGED = _(
	"Purge/Purging/Purged",
).split("/")
# NOTE: Verb Tenses are [ "Present/Present Participle/Past" ]
# NOTE: This ends up looking like [ "Install 20 Packages" ]
_INSTALL, _INSTALLING, _INSTALLED = _(
	"Install/Installing/Installed",
).split("/")
# NOTE: Verb Tenses are [ "Present/Present Participle/Past" ]
# NOTE: This ends up looking like [ "Reinstall 20 Packages" ]
_REINSTALL, _REINSTALLING, _REINSTALLED = _(
	"Reinstall/Reinstalling/Reinstalled",
).split("/")
# NOTE: Verb Tenses are [ "Present/Present Participle/Past" ]
# NOTE: This ends up looking like [ "Upgrade 20 Packages" ]
_UPGRADE, _UPGRADING, _UPGRADED = _(
	"Upgrade/Upgrading/Upgraded",
).split("/")
# NOTE: Verb Tenses are [ "Present/Present Participle/Past" ]
# NOTE: This ends up looking like [ "Downgrade 20 Packages" ]
_DOWNGRADE, _DOWNGRADING, _DOWNGRADED = _(
	"Downgrade/Downgrading/Downgraded",
).split("/")
# NOTE: Verb Tenses are [ "Present/Present Participle/Past" ]
# NOTE: This ends up looking like [ "Configure 20 Packages" ]
_CONFIGURE, _CONFIGURING, _CONFIGURED = _(
	"Configure/Configuring/Configured",
).split("/")

SUMMARY_LAYOUT = ("left_adjust", "right_adjust", "left_adjust")
UPGRADE_LAYOUT = ("pkg_blue", "old_version", "new_version", "pkg_size")
DOWNGRADE_LAYOUT = ("pkg_yellow", "old_version", "new_version", "pkg_size")
HELD_LAYOUT = ("pkg_yellow", "old_version", "new_version", "pkg_size")
DEFAULT_LAYOUT = ("pkg_green", "version", "pkg_size")
EXTRA_LAYOUT = ("pkg_magenta", "version", "pkg_size")
REMOVE_LAYOUT = ("pkg_red", "version", "pkg_size")

COLUMN_MAP: dict[str, dict[str, str | int]] = {
	"left_adjust": {"overflow": OVERFLOW},
	"right_adjust": {"justify": "right", "overflow": OVERFLOW},
	"pkg_green": {
		"header": f"{PACKAGE}:",
		"style": "bold green",
		"color": "GREEN",
		"overflow": OVERFLOW,
		"ratio": 2,
	},
	"pkg_red": {
		"header": f"{PACKAGE}:",
		"style": "bold red",
		"color": "RED",
		"overflow": OVERFLOW,
		"ratio": 2,
	},
	"pkg_blue": {
		"header": f"{PACKAGE}:",
		"style": "bold blue",
		"color": "BLUE",
		"overflow": OVERFLOW,
		"ratio": 3,
	},
	"pkg_yellow": {
		"header": f"{PACKAGE}:",
		"style": "bold orange_red1",
		"color": "YELLOW",
		"overflow": OVERFLOW,
		"ratio": 3,
	},
	"pkg_magenta": {
		"header": f"{PACKAGE}:",
		"style": "bold magenta",
		"color": "MAGENTA",
		"overflow": OVERFLOW,
		"ratio": 2,
	},
	"pkg_size": {
		"header": f"{SIZE}:",
		"justify": "right",
		"overflow": OVERFLOW,
		"ratio": 2,
	},
	"version": {"header": f"{VERSION}:", "overflow": "fold", "ratio": 2},
	"old_version": {"header": f"{OLD_VERSION}:", "overflow": "fold", "ratio": 2},
	"new_version": {"header": f"{NEW_VERSION}:", "overflow": "fold", "ratio": 2},
}
ROW_MAP: dict[str, str] = {
	"pkg_green": "name",
	"pkg_red": "name",
	"pkg_blue": "name",
	"pkg_yellow": "name",
	"pkg_magenta": "name",
	"pkg_size": "unit_size",
	"version": "version",
	"old_version": "old_version",
	"new_version": "version",
}


def get_columns(column_keys: Iterable[str]) -> Generator[Column, None, None]:
	"""Get the columns from our column map."""
	for key in column_keys:
		# Have to pop the color key to work with Column
		if "color" in (kwargs := COLUMN_MAP[key]):
			kwargs.pop("color")
		yield Column(**kwargs)  # type: ignore[arg-type]


def get_rows(pkg: NalaPackage, layout: Iterable[str]) -> Generator[Text, None, None]:
	"""Get the rows from our row map."""
	for key in layout:
		if key == "new_version":
			yield from_ansi(version_diff(pkg))
			continue
		if key == "color":
			continue
		yield from_ansi(getattr(pkg, ROW_MAP[key]))


def version_diff(pkg: NalaPackage) -> str:
	"""Return a colored diff of the new version."""
	if pkg.old_version:
		for i, char in enumerate(zip_longest(pkg.old_version, pkg.version)):
			if char[0] != char[1]:
				return f"{pkg.version[:i]}{color(pkg.version[i:], 'YELLOW')}"
	return pkg.version


@dataclass
class PackageHeaders:
	"""Tuple for package headers."""

	layout: tuple[str, ...]
	title: str
	summary: str = ""


@dataclass
class Headers:  # pylint: disable=too-many-instance-attributes
	"""Tuple for headers."""

	deleting: PackageHeaders
	auto_removing: PackageHeaders
	installing: PackageHeaders
	reinstalling: PackageHeaders
	upgrading: PackageHeaders
	downgrading: PackageHeaders
	configuring: PackageHeaders
	recommending: PackageHeaders
	suggesting: PackageHeaders
	held_pkgs: PackageHeaders
	# Packages that can be auto removed, but won't
	not_needed: PackageHeaders | None = None


def auto_remove_header(history: bool) -> tuple[str, str]:
	"""Get the auto remove header."""
	if history and arguments.is_purge():
		# NOTE: `/` will convert to a space for example in Irish it may be:
		# NOTE: "{package_total}/bpacáistí/a uasghrádú" -> "20 bpacáistí a uasghrádú"
		return _AUTO_PURGED, _AUTO_PURGED
	if history:
		return _AUTO_REMOVED, _AUTO_REMOVED
	if arguments.is_purge():
		return _AUTO_PURGING, _AUTO_PURGE
	return _AUTO_REMOVING, _AUTO_REMOVE


def remove_header(history: bool) -> tuple[str, str]:
	"""Get the remove header."""
	if history and arguments.is_purge():
		return _PURGED, _PURGED
	if history:
		return _REMOVED, _REMOVED
	return (_PURGING, _PURGE) if arguments.is_purge() else (_REMOVING, _REMOVE)


def get_headers() -> Headers:
	"""Get the headers for the summary table."""
	return Headers(
		PackageHeaders(REMOVE_LAYOUT, *remove_header(history=False)),
		PackageHeaders(REMOVE_LAYOUT, *auto_remove_header(history=False)),
		PackageHeaders(DEFAULT_LAYOUT, _INSTALLING, _INSTALL),
		PackageHeaders(DEFAULT_LAYOUT, _REINSTALLING, _REINSTALL),
		PackageHeaders(UPGRADE_LAYOUT, _UPGRADING, _UPGRADE),
		PackageHeaders(DOWNGRADE_LAYOUT, _DOWNGRADING, _DOWNGRADE),
		PackageHeaders(EXTRA_LAYOUT, _CONFIGURING, _CONFIGURE),
		PackageHeaders(EXTRA_LAYOUT, _("Recommended, Will Not Be Installed")),
		PackageHeaders(EXTRA_LAYOUT, _("Suggested, Will Not Be Installed")),
		PackageHeaders(
			DOWNGRADE_LAYOUT, _("Kept Back, Will Not Be Upgraded"), _("Kept Back")
		),
		PackageHeaders(REMOVE_LAYOUT, _("Auto-Removable, Will Not Be Removed")),
	)


def get_history_headers() -> Headers:
	"""Get the headers for the summary table."""
	return Headers(
		PackageHeaders(REMOVE_LAYOUT, *remove_header(history=True)),
		PackageHeaders(REMOVE_LAYOUT, *auto_remove_header(history=True)),
		PackageHeaders(DEFAULT_LAYOUT, _INSTALLED, _INSTALLED),
		PackageHeaders(DEFAULT_LAYOUT, _REINSTALLED, _REINSTALLED),
		PackageHeaders(UPGRADE_LAYOUT, _UPGRADED, _UPGRADED),
		PackageHeaders(DOWNGRADE_LAYOUT, _DOWNGRADED, _DOWNGRADED),
		PackageHeaders(EXTRA_LAYOUT, _CONFIGURED, _CONFIGURED),
		PackageHeaders(EXTRA_LAYOUT, _("Recommended, Will Not Be Installed")),
		PackageHeaders(
			DOWNGRADE_LAYOUT, _("Kept Back, Will Not Be Upgraded"), _("Kept Back")
		),
		PackageHeaders(EXTRA_LAYOUT, _("Suggested, Will Not Be Installed")),
	)


def gen_printers(
	nala_pkgs: PackageHandler, headers: Headers
) -> Generator[tuple[list[NalaPackage], PackageHeaders], None, None]:
	"""Generate the printers."""
	yield from (  # type: ignore[misc]
		(nala_pkgs.not_needed, headers.not_needed),
		(nala_pkgs.install_pkgs, headers.installing),
		(nala_pkgs.reinstall_pkgs, headers.reinstalling),
		(nala_pkgs.upgrade_pkgs, headers.upgrading),
		(nala_pkgs.downgrade_pkgs, headers.downgrading),
		(nala_pkgs.configure_pkgs, headers.configuring),
		(nala_pkgs.recommend_pkgs, headers.recommending),
		(nala_pkgs.suggest_pkgs, headers.suggesting),
		(nala_pkgs.held_pkgs, headers.held_pkgs),
		(
			nala_pkgs.autoremove_pkgs + nala_pkgs.autoremove_config,
			headers.auto_removing,
		),
		(nala_pkgs.delete_pkgs + nala_pkgs.delete_config, headers.deleting),
	)


def gen_package_table(
	nala_packages: list[NalaPackage] | list[NalaPackage | list[NalaPackage]],
	pkg_headers: PackageHeaders,
) -> Table:
	"""Print package transactions in a pretty format."""
	package_table = Table(
		*get_columns(pkg_headers.layout), padding=(0, 1), box=None, expand=True
	)

	# Add our packages
	or_deps: list[list[NalaPackage]] = []
	for pkg in nala_packages:
		if isinstance(pkg, list):
			or_deps.append(pkg)
			continue
		if pkg.old_version:
			package_table.add_row(*get_rows(pkg, pkg_headers.layout))
			continue
		package_table.add_row(*get_rows(pkg, pkg_headers.layout))

	# Add any or_deps
	for pkg in or_deps:
		package_table.add_row(*summary_or_depends(pkg))

	return package_table


def summary_or_depends(pkg: list[NalaPackage]) -> tuple[Tree, Group, Group]:
	"""Format Recommend and Suggests or dependencies."""
	pkg_tree = Tree(f"[default]{EITHER}[/default]", guide_style="default")
	_children = [pkg_tree.add(npkg.name) for npkg in pkg]
	return (
		pkg_tree,
		Group("", *(npkg.version for npkg in pkg)),
		Group("", *(npkg.unit_size for npkg in pkg)),
	)


def print_update_summary(nala_pkgs: PackageHandler, cache: Cache | None = None) -> None:
	"""Print our transaction summary."""
	dprint("Printing Update Summary")
	if arguments.simple_summary:
		print_short_summary(nala_pkgs, cache)
		return

	headers = get_headers() if cache else get_history_headers()

	main_table = Table.grid(expand=True)
	summary_header = Table(_("Summary"), padding=0, box=HORIZONTALS, expand=True)
	summary_table = Table.grid(*get_columns(SUMMARY_LAYOUT), padding=(0, 1))

	for pkg_set, header in gen_printers(nala_pkgs, headers):
		if not pkg_set:
			continue
		package_table = Table(header.title, padding=0, box=HORIZONTALS, expand=True)
		package_table.add_row(gen_package_table(pkg_set, header))
		main_table.add_row(package_table)
		# We don't need empty rows from these in the summary
		if nala_pkgs.no_summary(pkg_set):
			continue
		# NOTE: This ends up looking like [ "Configure 20 Packages" ]
		summary_table.add_row(header.summary, f"{len(pkg_set)}", _("Packages"))

	summary_header.add_row(summary_table)
	main_table.add_row(summary_header)
	console.print(main_table)

	if cache:
		footer_table = Table(
			*get_columns(SUMMARY_LAYOUT), box=None, show_footer=True, show_header=False
		)
		if (download := cache.required_download) > 0:
			footer_table.add_row(_("Total download size"), unit_str(download))
		if (space := cache.required_space) < 0:
			footer_table.add_row(_("Disk space to free"), unit_str(-space))
		if space > 0:
			footer_table.add_row(_("Disk space required"), unit_str(space))
		console.print(footer_table)

	if cache and arguments.download_only:
		print(_("Nala will only download the packages"))


def append_or_print(string: str, pkg_name: str) -> bool:
	"""Print the string and return False, or return True."""
	string_size = len(from_ansi(string))
	pkg_name_size = len(from_ansi(pkg_name))

	# Check the string size plus a little buffer
	# against the available terminal width
	if string_size + pkg_name_size + 3 >= term.columns:
		dprint(
			f"Line Buffer Full: {string_size + pkg_name_size + 3},"
			f" Terminal Width: {term.columns}; Printing Buffer"
		)
		return False
	return True


def format_pkgs(
	pkg_set: list[NalaPackage] | list[NalaPackage | list[NalaPackage]],
	pkg_color: str,
) -> None:
	"""Format the packages in a simple way."""
	pkg_string = ""
	final_i = len(pkg_set) - 1
	for i, pkg in enumerate(pkg_set):
		strip_comma = final_i == i

		# We don't want to add the space at the start of the string
		if pkg_string:
			pkg_string += " "

		if isinstance(pkg, list):
			next_string = f"[ {' | '.join(pkg.name for pkg in pkg)} ],"
			if append_or_print(pkg_string, next_string):
				pkg_string += next_string
			if not strip_comma:
				continue

		if isinstance(pkg, NalaPackage) and append_or_print(pkg_string, pkg.name):
			pkg_string += f"{color(pkg.name, pkg_color)},"
			if not strip_comma:
				continue

		dprint(
			f"Current Index: {i}, Final Index: {final_i}, Strip Comma: {strip_comma}"
		)
		print(f"  {pkg_string.strip(', ') if strip_comma else pkg_string}")
		pkg_string = ""


def print_short_summary(nala_pkgs: PackageHandler, cache: Cache | None = None) -> None:
	"""Print our transaction summary."""
	dprint("Printing Update Summary")
	headers = get_headers() if cache else get_history_headers()

	summary_table = []
	for pkg_set, header in gen_printers(nala_pkgs, headers):
		if not pkg_set or not header:
			continue

		dprint(f"{header.title}: {pkg_set}")
		# Not sure on this formatting yet
		print(color(f"{header.title}:"))
		# The color should be the first index of our layout.
		# This might be "pkg_magenta". And then get the nala 'color' key
		# Ensure it is a string with a quick cast. There could be ints
		pkg_color = f"{COLUMN_MAP[header.layout[0]]['color']}"
		format_pkgs(pkg_set, pkg_color)

		# No summary is needed for this one.
		if nala_pkgs.no_summary(pkg_set):
			continue
		# NOTE: This ends up looking like [ "Configure 20 Packages" ]
		summary_table.append(f"{color(header.summary, pkg_color)} {len(pkg_set)}")

	# Print the summary
	if summary_table:
		summary = color(_("Summary"))
		print(f"{summary}:\n  {', '.join(summary_table)}")

	if cache:
		cache_string = "  "
		if (download := cache.required_download) > 0:
			cache_string += color(_("Total download size")) + f" {unit_str(download)}, "
		if (space := cache.required_space) < 0:
			cache_string += color(_("Disk space to free")) + f" {unit_str(-space)}, "
		if space > 0:
			cache_string += color(_("Disk space required")) + f" {unit_str(space)}, "
		print(cache_string.rstrip(", "))

	if cache and arguments.download_only:
		print(_("Nala will only download the packages"))
