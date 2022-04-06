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
from typing import Iterable, Union, cast

import jsbeautifier
import typer
from apt.package import Package

from nala import _, color
from nala.cache import Cache
from nala.constants import ERROR_PREFIX, JSON_OPTIONS, NALA_HISTORY, WARNING_PREFIX
from nala.options import (
	ASSUME_YES,
	AUTO_REMOVE,
	DEBUG,
	DOWNLOAD_ONLY,
	FIX_BROKEN,
	OPTION,
	PURGE,
	RAW_DPKG,
	RECOMMENDS,
	REMOVE_ESSENTIAL,
	SUGGESTS,
	UPDATE,
	VERBOSE,
	_doc,
	arguments,
	history_typer,
)
from nala.rich import Column, Table
from nala.utils import (
	DelayedKeyboardInterrupt,
	NalaPackage,
	PackageHandler,
	ask,
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

HistoryFile = dict[str, dict[str, Union[str, bool, list[str], list[list[str]]]]]
HistoryEntry = dict[str, Union[str, bool, list[str], list[list[str]]]]

NOT_SUPPORTED = _(
	"{error} '{command}' for operations other than install or remove are not currently supported"
)


def load_history_file() -> HistoryFile:
	"""Load Nala history."""
	try:
		return cast(
			HistoryFile,
			json.loads(NALA_HISTORY.read_text(encoding="utf-8")),
		)
	except JSONDecodeError:
		sys.exit(
			_(
				"{error} History file seems corrupt. You should try removing {file}"
			).format(error=ERROR_PREFIX, file=NALA_HISTORY)
		)


def write_history_file(data: HistoryFile) -> None:
	"""Write history to file."""
	with DelayedKeyboardInterrupt():
		with open(NALA_HISTORY, "w", encoding="utf-8") as file:
			file.write(jsbeautifier.beautify(json.dumps(data), JSON_OPTIONS))


@_doc
@history_typer.callback(invoke_without_command=True)
def history_summary(ctx: typer.Context) -> None:
	"""Show transaction history.

	Running `nala history` with no subcommands prints an overview of all translations.
	"""
	if ctx.invoked_subcommand:
		return

	if not NALA_HISTORY.exists():
		sys.exit(_("{error} No history exists.").format(error=ERROR_PREFIX))
	history_file = load_history_file()
	pop_nala(history_file)

	names: list[Iterable[str]] = []

	for key, entry in history_file.items():
		command = get_list(entry, "Command")
		if command[0] in ("update", "upgrade"):
			for pkg in get_nala_packages(entry, "Upgraded"):
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
	if not names:
		sys.exit(_("{error} No history exists.").format(error=ERROR_PREFIX))

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


def get_nala_packages(hist_entry: HistoryEntry, key: str) -> list[NalaPackage]:
	"""Type cast history package is list of lists."""
	nala_pkgs = []
	for pkg_list in get_packages(hist_entry, key):
		dprint(f"{key} List: {pkg_list}")
		if len(pkg_list) == 4:
			try:
				name, new_version, size, old_version = pkg_list
				nala_pkgs.append(NalaPackage(name, new_version, int(size), old_version))
			except ValueError:
				name, old_version, new_version, size = pkg_list
				nala_pkgs.append(NalaPackage(name, new_version, int(size), old_version))
			continue
		name, new_version, size = pkg_list
		nala_pkgs.append(NalaPackage(name, new_version, int(size)))
	return nala_pkgs


def get_packages(hist_entry: HistoryEntry, key: str) -> list[list[str]]:
	"""Type cast history packages is list of strings."""
	return cast(list[list[str]], hist_entry.get(key, [[]]))


def get_list(hist_entry: HistoryEntry, key: str) -> list[str]:
	"""Type cast history command is list of strings."""
	return cast(list[str], hist_entry.get(key, []))


def get_bool(hist_entry: HistoryEntry, key: str) -> bool:
	"""Type cast history entry is bool."""
	return cast(bool, hist_entry.get(key, False))


def get_str(hist_entry: HistoryEntry, key: str) -> str:
	"""Type cast history entry is str."""
	return cast(str, hist_entry.get(key, ""))


def pop_nala(hist_file: HistoryFile) -> HistoryEntry:
	"""Pop the Nala field from the history."""
	return hist_file.pop("Nala", {})


@_doc
@history_typer.command("info")
def history_info(
	_hist_id: int = typer.Argument(
		..., metavar="ID", help=_("Transaction number to show info about")
	),
) -> None:
	"""Show information about a specific transaction."""
	hist_id = str(_hist_id)
	dprint(f"History info {hist_id}")
	hist_entry = get_history(hist_id)
	dprint(f"History Entry: {hist_entry}")
	arguments.purge = get_bool(hist_entry, "Purged")
	nala_pkgs = PackageHandler()
	nala_pkgs.autoremove_pkgs = get_nala_packages(hist_entry, "Auto-Removed")
	nala_pkgs.delete_pkgs = get_nala_packages(hist_entry, "Removed")
	nala_pkgs.install_pkgs = get_nala_packages(hist_entry, "Installed")
	nala_pkgs.reinstall_pkgs = get_nala_packages(hist_entry, "Reinstalled")
	nala_pkgs.upgrade_pkgs = get_nala_packages(hist_entry, "Upgraded")
	nala_pkgs.downgrade_pkgs = get_nala_packages(hist_entry, "Downgraded")

	print_update_summary(nala_pkgs)


def history_sudo(
	redo: bool = False,
	clear: bool = False,
) -> None:
	"""Check if we need sudo."""
	if not term.is_su():
		if clear:
			sys.exit(" ".join([ERROR_PREFIX, _("Nala needs root to clear history")]))
		if redo:
			sys.exit(" ".join([ERROR_PREFIX, _("Nala needs root to redo history")]))
		sys.exit(" ".join([ERROR_PREFIX, _("Nala needs root to undo history")]))


def unlink_history(value: bool) -> None:
	"""Remove the history file."""
	history_sudo(clear=True)
	if not value:
		return
	dprint("History clear all")

	history_dict = {"Nala": get_history("Nala")}
	NALA_HISTORY.unlink(missing_ok=True)
	write_history_file(history_dict)

	print(_("History has been cleared"))
	sys.exit()


@_doc
@history_typer.command("clear")
def history_clear(
	_hist_id: int = typer.Argument(
		..., metavar="ID", help=_("Transaction number to clear")
	),
	_all: bool = typer.Option(  # pylint: disable=unused-argument
		False, "--all", callback=unlink_history, help=_("Clear the entire history.")
	),
) -> None:
	"""Clear a transaction or the entire history."""
	hist_id = str(_hist_id)
	dprint(f"History clear {hist_id}")
	if not NALA_HISTORY.exists():
		eprint(_("No history exists to clear..."))
		return

	if hist_id not in (history_file := load_history_file()).keys():
		sys.exit(
			_("{error} ID: {hist_id} does not exist in the history").format(
				error=ERROR_PREFIX, hist_id=color(hist_id, "YELLOW")
			)
		)
	history_edit: HistoryFile = {}
	nala_dict = pop_nala(history_file)
	num = 0
	# Using sum increments to relabled the IDs so when you remove just one
	# There isn't a gap in ID numbers and it looks concurrent.
	for key, value in history_file.items():
		if key != hist_id:
			num += 1
			history_edit[str(num)] = value
	history_edit["Nala"] = nala_dict
	print(_("History has been altered..."))
	write_history_file(history_edit)


@_doc
@history_typer.command("undo", help=_("Undo a transaction."))
@history_typer.command("redo", help=_("Redo a transaction."))
# pylint: disable=unused-argument,too-many-arguments,too-many-locals
def history_undo(
	ctx: typer.Context,
	hist_id: str = typer.Argument(..., metavar="ID", help=_("Transaction number")),
	purge: bool = PURGE,
	debug: bool = DEBUG,
	raw_dpkg: bool = RAW_DPKG,
	download_only: bool = DOWNLOAD_ONLY,
	remove_essential: bool = REMOVE_ESSENTIAL,
	update: bool = UPDATE,
	auto_remove: bool = AUTO_REMOVE,
	install_recommends: bool = RECOMMENDS,
	install_suggests: bool = SUGGESTS,
	fix_broken: bool = FIX_BROKEN,
	assume_yes: bool = ASSUME_YES,
	dpkg_option: list[str] = OPTION,
	verbose: bool = VERBOSE,
) -> None:
	"""History undo/redo commands."""
	from nala.nala import (  # pylint: disable=cyclic-import, import-outside-toplevel
		_install,
		_remove,
	)

	arguments.history = ctx.command.name
	arguments.history_id = hist_id
	redo = ctx.command.name == "redo"
	history_sudo(redo=redo)

	dprint(f"History: {ctx.command.name} {hist_id}")

	transaction = get_history(hist_id)
	dprint(f"Transaction: {transaction}")

	if not arguments.purge:
		if purge := get_bool(transaction, "Purged"):
			eprint(
				_("{warn} This history entry was a purge.").format(warn=WARNING_PREFIX)
			)
			if ask(_("Do you want to continue with purge enabled?")):
				arguments.purge = purge

	explicit = get_list(transaction, "Explicit")
	operation = get_str(transaction, "Operation")
	command = get_list(transaction, "Command")[0]

	if operation == "remove" or command in (
		"remove",
		"purge",
		"autoremove",
		"autopurge",
	):
		pkgs = [pkg[0] for pkg in get_packages(transaction, "Removed")]
		pkgs.extend([pkg[0] for pkg in get_packages(transaction, "Auto-Removed")])

		if redo:
			_remove(pkgs)
			return
		_install(explicit or pkgs, ctx)
		return

	if operation == "install" or command == "install":
		pkgs = [pkg[0] for pkg in get_packages(transaction, "Installed")]
		if redo:
			_install(explicit or pkgs, ctx)
			return
		_remove(pkgs)
		return
	sys.exit(
		NOT_SUPPORTED.format(
			error=ERROR_PREFIX, command=f"{arguments.command} {ctx.command.name}"
		)
	)


def set_user_installed(
	cache: Cache, user_explicit: list[Package], user_installed: set[str]
) -> None:
	"""Set the User-Installed field."""
	if user_explicit:
		for pkg in user_explicit:
			if pkg.marked_install:
				user_installed.add(pkg.name)
			if pkg.marked_delete:
				user_installed.discard(pkg.name)

	for pkg_name in user_installed:
		if pkg_name not in cache:
			user_installed.discard(pkg_name)
		pkg = cache[pkg_name]
		if pkg.installed or pkg.marked_install:
			continue
		user_installed.discard(pkg.name)


def write_history(cache: Cache, handler: PackageHandler, operation: str) -> None:
	"""Prepare history for writing."""
	history_dict = load_history_file() if NALA_HISTORY.exists() else {}
	nala_dict = pop_nala(history_dict)
	user_installed = set(get_list(nala_dict, "User-Installed"))

	hist_id = str(len(history_dict) + 1 if history_dict else 1)
	altered = (
		handler.delete_total
		+ handler.autoremove_total
		+ handler.install_total
		+ handler.upgrade_total
		+ handler.reinstall_total
		+ handler.downgrade_total
	)

	set_user_installed(cache, handler.user_explicit, user_installed)

	transaction: HistoryEntry = {
		"Date": get_date(),
		"Requested-By": f"{USER} ({UID})",
		"Command": sys.argv[1:],
		"Altered": str(altered),
		"Purged": arguments.is_purge(),
		"Operation": operation,
		"Explicit": [pkg.name for pkg in handler.user_explicit],
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
	nala_dict["History-Version"] = "1"
	nala_dict["User-Installed"] = list(user_installed)
	history_dict["Nala"] = nala_dict
	write_history_file(history_dict)


def get_history(hist_id: str) -> HistoryEntry:
	"""Get the history from file."""
	dprint(f"Getting history {hist_id}")
	if not NALA_HISTORY.exists():
		sys.exit(_("{error} No history exists...").format(error=ERROR_PREFIX))
	if transaction := load_history_file().get(hist_id):
		return transaction
	sys.exit(
		_("{error} Transaction {num} doesn't exist.").format(
			error=ERROR_PREFIX, num=hist_id
		)
	)
