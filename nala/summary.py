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
"""Module for printing the transaction summary."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Generator

from nala import _, console
from nala.cache import Cache
from nala.options import arguments
from nala.rich import HORIZONTALS, OVERFLOW, Column, Group, Table, Tree
from nala.utils import NalaPackage, PackageHandler, dprint, unit_str

# NOTE: Everything in the `summary.py` module is for the transaction summary.
# NOTE: The following are the headers for the transaction summary.
PACKAGE, VERSION, SIZE, OLD_VERSION, NEW_VERSION, EITHER = _(
	"Package:/Version:/Size:/Old Version:/New Version:/Either:"
).split("/")

# NOTE: This is the summary layout for "Upgrade 20 Packages" messages.
# NOTE: right_adjust should be placed where the package_total will be.
# NOTE: All formatting strings MUST remain the same but can be reordered.
# NOTE: This is used for strings like "Auto-Purged/{package_total}/Packages".
# NOTE: `/` will convert to a space. For example, in Irish. it may be:
# NOTE: "{package_total}/bpacáistí/a uasghrádú" -> "20 bpacáistí a uasghrádú"
SUMMARY_LAYOUT = _("left_adjust right_adjust left_adjust").split()
# NOTE: Upgrading Columns layout
UPGRADE_LAYOUT = _("pkg_blue old_version new_version pkg_size").split()
# NOTE: Downgrading Columns layout
DOWNGRADE_LAYOUT = _("pkg_yellow old_version new_version pkg_size").split()
# NOTE: The Default Columns layout
DEFAULT_LAYOUT = _("pkg_green version pkg_size").split()
# NOTE: Recommend and Suggests Columns layout
EXTRA_LAYOUT = _("pkg_magenta version pkg_size").split()
# NOTE: Removing Columns layout
REMOVE_LAYOUT = _("pkg_red version pkg_size").split()

COLUMN_MAP: dict[str, dict[str, str | int]] = {
	"left_adjust": {"overflow": OVERFLOW},
	"right_adjust": {"justify": "right", "overflow": OVERFLOW},
	"pkg_green": {
		"header": PACKAGE,
		"style": "bold green",
		"overflow": OVERFLOW,
		"ratio": 2,
	},
	"pkg_red": {
		"header": PACKAGE,
		"style": "bold red",
		"overflow": OVERFLOW,
		"ratio": 2,
	},
	"pkg_blue": {
		"header": PACKAGE,
		"style": "bold blue",
		"overflow": OVERFLOW,
		"ratio": 3,
	},
	"pkg_yellow": {
		"header": PACKAGE,
		"style": "bold orange_red1",
		"overflow": OVERFLOW,
		"ratio": 2,
	},
	"pkg_magenta": {
		"header": PACKAGE,
		"style": "bold magenta",
		"overflow": OVERFLOW,
		"ratio": 2,
	},
	"pkg_size": {"header": SIZE, "justify": "right", "overflow": OVERFLOW, "ratio": 2},
	"version": {"header": VERSION, "overflow": "fold", "ratio": 2},
	"old_version": {"header": OLD_VERSION, "overflow": "fold", "ratio": 2},
	"new_version": {"header": NEW_VERSION, "overflow": "fold", "ratio": 2},
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


def get_columns(column_keys: list[str]) -> Generator[Column, None, None]:
	"""Get the columns from our column map."""
	for key in column_keys:
		yield Column(**COLUMN_MAP[key])  # type: ignore[arg-type]


def get_rows(pkg: NalaPackage, layout: list[str]) -> Generator[str, None, None]:
	"""Get the rows from our row map."""
	for key in layout:
		yield getattr(pkg, ROW_MAP[key])


@dataclass
class PackageHeaders:
	"""Tuple for package headers."""

	layout: list[str]
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


def auto_remove_header(history: bool) -> tuple[str, str]:
	"""Get the auto remove header."""
	if history and arguments.is_purge():
		# NOTE: `/` will convert to a space for example in Irish it may be:
		# NOTE: "{package_total}/bpacáistí/a uasghrádú" -> "20 bpacáistí a uasghrádú"
		return _("Auto-Purged"), _("Auto-Purged/{package_total}/Packages")
	if history:
		return _("Auto-Removed"), _("Auto-Removed/{package_total}/Packages")
	if arguments.is_purge():
		return _("Auto-Purging"), _("Auto-Purge/{package_total}/Packages")
	return _("Auto-Removing"), _("Auto-Remove/{package_total}/Packages")


def remove_header(history: bool) -> tuple[str, str]:
	"""Get the remove header."""
	if history and arguments.is_purge():
		return _("Purged"), _("Purged/{package_total}/Packages")
	if history:
		return _("Removed"), _("Removed/{package_total}/Packages")
	if arguments.is_purge():
		return _("Purging"), _("Purge/{package_total}/Packages")
	return _("Removing"), _("Remove/{package_total}/Packages")


def get_headers() -> Headers:
	"""Get the headers for the summary table."""
	return Headers(
		PackageHeaders(REMOVE_LAYOUT, *remove_header(history=False)),
		PackageHeaders(REMOVE_LAYOUT, *auto_remove_header(history=False)),
		PackageHeaders(
			DEFAULT_LAYOUT, _("Installing"), _("Install/{package_total}/Packages")
		),
		PackageHeaders(
			DEFAULT_LAYOUT, _("Reinstalling"), _("Reinstall/{package_total}/Packages")
		),
		PackageHeaders(
			UPGRADE_LAYOUT, _("Upgrading"), _("Upgrade/{package_total}/Packages")
		),
		PackageHeaders(
			DOWNGRADE_LAYOUT, _("Downgrading"), _("Downgrade/{package_total}/Packages")
		),
		PackageHeaders(
			EXTRA_LAYOUT, _("Configuring"), _("Configure/{package_total}/Packages")
		),
		PackageHeaders(EXTRA_LAYOUT, _("Recommended, Will Not Be Installed")),
		PackageHeaders(EXTRA_LAYOUT, _("Suggested, Will Not Be Installed")),
	)


def get_history_headers() -> Headers:
	"""Get the headers for the summary table."""
	return Headers(
		PackageHeaders(REMOVE_LAYOUT, *remove_header(history=True)),
		PackageHeaders(REMOVE_LAYOUT, *auto_remove_header(history=True)),
		PackageHeaders(
			DEFAULT_LAYOUT, _("Installed"), _("Installed/{package_total}/Packages")
		),
		PackageHeaders(
			DEFAULT_LAYOUT, _("Reinstalled"), _("Reinstalled/{package_total}/Packages")
		),
		PackageHeaders(
			UPGRADE_LAYOUT, ("Upgraded"), _("Upgraded/{package_total}/Packages")
		),
		PackageHeaders(
			DOWNGRADE_LAYOUT, _("Downgraded"), _("Downgraded/{package_total}/Packages")
		),
		PackageHeaders(
			EXTRA_LAYOUT, _("Configured"), _("Configured/{package_total}/Packages")
		),
		PackageHeaders(EXTRA_LAYOUT, _("Recommended, Will Not Be Installed")),
		PackageHeaders(EXTRA_LAYOUT, _("Suggested, Will Not Be Installed")),
	)


def gen_printers(
	nala_pkgs: PackageHandler, headers: Headers
) -> Generator[tuple[list[NalaPackage], PackageHeaders], None, None]:
	"""Generate the printers."""
	yield from (  # type: ignore[misc]
		(nala_pkgs.delete_pkgs, headers.deleting),
		(nala_pkgs.autoremove_pkgs, headers.auto_removing),
		(nala_pkgs.install_pkgs, headers.installing),
		(nala_pkgs.reinstall_pkgs, headers.reinstalling),
		(nala_pkgs.upgrade_pkgs, headers.upgrading),
		(nala_pkgs.downgrade_pkgs, headers.downgrading),
		(nala_pkgs.configure_pkgs, headers.configuring),
		(nala_pkgs.recommend_pkgs, headers.recommending),
		(nala_pkgs.suggest_pkgs, headers.suggesting),
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
		summary_table.add_row(
			*header.summary.format(package_total=len(pkg_set)).split("/")
		)

	summary_header.add_row(summary_table)
	main_table.add_row(summary_header)

	if cache:
		footer_table = Table.grid()
		if (download := cache.required_download) > 0:
			footer_table.add_row(_("Total download size"), unit_str(download))
		if (space := cache.required_space) < 0:
			footer_table.add_row(_("Disk space to free"), unit_str(-space))
		if space > 0:
			footer_table.add_row(_("Disk space required"), unit_str(space))
		if footer_table.columns:
			summary_header.add_row(footer_table)

	console.print(main_table)
	if cache and arguments.download_only:
		print(_("Nala will only download the packages"))
