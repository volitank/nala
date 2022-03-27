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
"""Functions for handling Nala History."""
from __future__ import annotations

import json
import sys
from getpass import getuser
from json.decoder import JSONDecodeError
from os import environ, getuid
from pwd import getpwnam
from typing import Iterable

import jsbeautifier

from nala import _, color
from nala.constants import ERROR_PREFIX, JSON_OPTIONS, NALA_HISTORY
from nala.options import arguments
from nala.rich import Column, Table
from nala.utils import (
	DelayedKeyboardInterrupt,
	NalaPackage,
	PackageHandler,
	dprint,
	eprint,
	get_date,
	print_update_summary,
	term,
)

USER: str = environ.get("DOAS_USER", "")
UID: int = 0
if USER:
	UID = getpwnam(USER).pw_uid
else:
	USER = environ.get("SUDO_USER", getuser())
	UID = int(environ.get("SUDO_UID", getuid()))

NOT_SUPPORTED = _(
	"{error} '{command}' for operations other than install or remove are not currently supported"
)


def load_history_file() -> dict[str, dict[str, str | list[str] | list[list[str]]]]:
	"""Load Nala history."""
	try:
		check = json.loads(NALA_HISTORY.read_text(encoding="utf-8"))
		assert isinstance(check, dict)
		return check
	except JSONDecodeError:
		sys.exit(
			_(
				"{error} History file seems corrupt. You should try removing {file}"
			).format(error=ERROR_PREFIX, file=NALA_HISTORY)
		)


def write_history_file(
	data: dict[str, dict[str, str | list[str] | list[list[str]]]]
) -> None:
	"""Write history to file."""
	with DelayedKeyboardInterrupt():
		with open(NALA_HISTORY, "w", encoding="utf-8") as file:
			file.write(jsbeautifier.beautify(json.dumps(data), JSON_OPTIONS))


def history_summary() -> None:
	"""History command."""
	if not NALA_HISTORY.exists():
		eprint(_("{error} No history exists...").format(error=ERROR_PREFIX))
		return
	history_file = load_history_file()
	names: list[Iterable[str]] = []

	for key, entry in history_file.items():
		command = get_hist_list(entry, "Command")
		if command[0] in ("update", "upgrade"):
			for pkg in get_hist_package(entry, "Upgraded"):
				command.append(pkg.name)

		names.append(
			map(
				str,
				(
					key,
					" ".join(command),
					entry.get("Date"),
					entry.get("Altered"),
					entry.get("Requested-By"),
				),
			)
		)

	max_width = term.columns - 69
	history_table = Table(
		Column("ID"),
		Column("Command", no_wrap=True, max_width=max_width, overflow=term.overflow),
		Column("Date and Time", no_wrap=True),
		Column("Altered", justify="right"),
		Column("Requested-By"),
		padding=(0, 2),
		box=None,
	)

	for item in names:
		history_table.add_row(*item)
	term.console.print(history_table)


def get_hist_package(
	hist_entry: dict[str, str | list[str] | list[list[str]]], key: str
) -> list[NalaPackage]:
	"""Type enforce history package is list of lists."""
	nala_pkgs = []
	for pkg_list in hist_entry.get(key, []):
		if isinstance(pkg_list, list):
			dprint(f"{key} List: {pkg_list}")
			if len(pkg_list) == 4:
				(
					name,
					new_version,
					size,
					old_version,
				) = pkg_list
				nala_pkgs.append(NalaPackage(name, new_version, int(size), old_version))
				continue
			name, new_version, size = pkg_list
			nala_pkgs.append(NalaPackage(name, new_version, int(size)))
	return nala_pkgs


def get_hist_list(
	hist_entry: dict[str, str | list[str] | list[list[str]]], key: str
) -> list[str]:
	"""Type enforce history package is list of strings."""
	return [pkg for pkg in hist_entry[key] if isinstance(pkg, str)]


def history_info(hist_id: str) -> None:
	"""History info command."""
	dprint(f"History info {hist_id}")
	hist_entry = get_history(hist_id)
	dprint(f"History Entry: {hist_entry}")
	nala_pkgs = PackageHandler()
	nala_pkgs.autoremove_pkgs = get_hist_package(hist_entry, "Auto-Removed")
	nala_pkgs.delete_pkgs = get_hist_package(hist_entry, "Removed")
	nala_pkgs.install_pkgs = get_hist_package(hist_entry, "Installed")
	nala_pkgs.reinstall_pkgs = get_hist_package(hist_entry, "Reinstalled")
	nala_pkgs.upgrade_pkgs = get_hist_package(hist_entry, "Upgraded")
	nala_pkgs.downgrade_pkgs = get_hist_package(hist_entry, "Downgraded")

	print_update_summary(nala_pkgs)


def history_clear(hist_id: str) -> None:
	"""Show command."""
	dprint(f"History clear {hist_id}")
	if not NALA_HISTORY.exists():
		eprint(_("No history exists to clear..."))
		return

	if hist_id == "all":
		NALA_HISTORY.unlink()
		print(_("History has been cleared"))
		return

	if hist_id not in (history_file := load_history_file()).keys():
		sys.exit(
			_("{error} ID: {hist_id} does not exist in the history").format(
				error=ERROR_PREFIX, hist_id=color(hist_id, "YELLOW")
			)
		)
	history_edit: dict[str, dict[str, str | list[str] | list[list[str]]]] = {}
	num = 0
	# Using sum increments to relabled the IDs so when you remove just one
	# There isn't a gap in ID numbers and it looks concurrent.
	for key, value in history_file.items():
		if key != hist_id:
			num += 1
			history_edit[str(num)] = value
	print(_("History has been altered..."))
	write_history_file(history_edit)


def history_undo(hist_id: str, redo: bool = False) -> None:
	"""History undo/redo commands."""
	from nala.nala import (  # pylint: disable=cyclic-import, import-outside-toplevel
		install,
		remove,
	)

	dprint(f"History: {arguments.mode} {hist_id}")

	transaction = get_history(hist_id)
	dprint(f"Transaction: {transaction}")

	command = transaction.get("Command", [""])[0]
	if command == "remove":
		pkgs = [str(pkg[0]) for pkg in transaction.get("Removed", [])]
		pkgs.extend([str(pkg[0]) for pkg in transaction.get("Auto-Removed", [])])

		if redo:
			remove(pkgs)
			return
		install(pkgs)
		return

	if command == "install":
		pkgs = [str(pkg[0]) for pkg in transaction.get("Installed", [])]
		if redo:
			install(pkgs)
			return
		remove(pkgs)
		return
	sys.exit(
		NOT_SUPPORTED.format(
			error=ERROR_PREFIX, command=f"{arguments.command} {arguments.mode}"
		)
	)


def write_history(handler: PackageHandler) -> None:
	"""Prepare history for writing."""
	history_dict = load_history_file() if NALA_HISTORY.exists() else {}
	hist_id = str(len(history_dict) + 1 if history_dict else 1)
	altered = (
		handler.delete_total
		+ handler.autoremove_total
		+ handler.install_total
		+ handler.upgrade_total
		+ handler.reinstall_total
		+ handler.downgrade_total
	)

	transaction: dict[str, str | list[str] | list[list[str]]] = {
		"Date": get_date(),
		"Requested-By": f"{USER} ({UID})",
		"Command": sys.argv[1:],
		"Altered": str(altered),
		"Removed": [
			[pkg.name, pkg.version, str(pkg.size)] for pkg in handler.delete_pkgs
		],
		"Auto-Removed": [
			[pkg.name, pkg.version, str(pkg.size)] for pkg in handler.autoremove_pkgs
		],
		"Installed": [
			[pkg.name, pkg.version, str(pkg.size)] for pkg in handler.install_pkgs
		],
		"Reinstalled": [
			[pkg.name, pkg.version, str(pkg.size)] for pkg in handler.reinstall_pkgs
		],
		"Upgraded": [
			[pkg.name, pkg.version, str(pkg.size), str(pkg.old_version)]
			for pkg in handler.upgrade_pkgs
		],
		"Downgraded": [
			[pkg.name, pkg.version, str(pkg.size), str(pkg.old_version)]
			for pkg in handler.downgrade_pkgs
		],
	}

	history_dict[hist_id] = transaction
	write_history_file(history_dict)


def get_history(hist_id: str) -> dict[str, str | list[str] | list[list[str]]]:
	"""Get the history from file."""
	dprint(f"Getting history {hist_id}")
	if not NALA_HISTORY.exists():
		sys.exit(_("{error} No history exists...").format(error=ERROR_PREFIX))
	history_file: dict[str, dict[str, str | list[str] | list[list[str]]]] = json.loads(
		NALA_HISTORY.read_text(encoding="utf-8")
	)
	if transaction := history_file.get(hist_id):
		return transaction
	sys.exit(
		_("{error} Transaction {num} doesn't exist.").format(
			error=ERROR_PREFIX, num=hist_id
		)
	)
