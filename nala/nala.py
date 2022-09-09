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
"""Main module for Nala which facilitates apt."""
from __future__ import annotations

import fnmatch
import re
import sys
from subprocess import run
from typing import Generator, List, Optional, Pattern

import apt_pkg
import typer
from apt.package import Package, Version

from nala import _, color
from nala.cache import Cache
from nala.constants import (
	ARCHIVE_DIR,
	CAT,
	DPKG_STATE,
	ERROR_PREFIX,
	LISTS_DIR,
	LISTS_PARTIAL_DIR,
	NALA_SOURCES,
	PARTIAL_DIR,
	PKGCACHE,
	SRCPKGCACHE,
)
from nala.error import BrokenError, pkg_error
from nala.history import get_history, get_list
from nala.install import (
	auto_remover,
	check_broken,
	check_state,
	check_term_ask,
	fix_excluded,
	get_changes,
	install_local,
	package_manager,
	setup_cache,
	split_local,
)
from nala.options import (
	ALL_ARCHES,
	ALL_VERSIONS,
	ASSUME_YES,
	AUTO_REMOVE,
	CONFIG,
	DEBUG,
	DOWNLOAD_ONLY,
	FETCH,
	FIX_BROKEN,
	FULL,
	INSTALLED,
	LISTS,
	MAN_HELP,
	NALA_INSTALLED,
	NAMES,
	OPTION,
	PURGE,
	RAW_DPKG,
	RECOMMENDS,
	REMOVE_ESSENTIAL,
	SUGGESTS,
	UPDATE,
	UPGRADABLE,
	UPGRADEABLE,
	VERBOSE,
	VIRTUAL,
	arguments,
	nala,
)
from nala.rich import ELLIPSIS
from nala.search import iter_search, search_name
from nala.show import additional_notice, pkg_not_found, show_main
from nala.utils import (
	PackageHandler,
	ask,
	command_help,
	compile_regex,
	dedupe_list,
	eprint,
	get_version,
	iter_remove,
	sudo_check,
	vprint,
)

nala_pkgs = PackageHandler()


def _fix_broken(nested_cache: Cache | None = None) -> None:
	"""Attempt to fix broken packages, if any."""
	cache = nested_cache or setup_cache()
	print("Fixing Broken Packages...")
	cache.fix_broken()

	if nested_cache:
		print(color(_("There are broken packages that need to be fixed!"), "YELLOW"))
		print(
			_("You can use {switch} if you'd like to try without fixing them.").format(
				switch=color("--no-fix-broken", "YELLOW")
			)
		)
	else:
		check_state(cache, nala_pkgs)
	get_changes(cache, nala_pkgs, "fix-broken")


def _remove(pkg_names: list[str]) -> None:
	sudo_check()

	cache = setup_cache()
	check_state(cache, nala_pkgs)

	pkg_names = cache.glob_filter(dedupe_list(pkg_names))
	pkg_names = cache.virtual_filter(pkg_names, remove=True)
	broken, not_found, ver_failed = check_broken(
		pkg_names,
		cache,
		remove=True,
	)

	for pkg_name in not_found[:]:
		if cache.is_any_virtual(pkg_name):
			not_found.remove(pkg_name)
			pkg_names.remove(pkg_name)

	if not_found or ver_failed:
		pkg_error(not_found, cache)

	nala_pkgs.user_explicit = [cache[pkg_name] for pkg_name in pkg_names]
	if not package_manager(pkg_names, cache, remove=True):
		BrokenError(cache).broken_remove(broken)

	auto_remover(cache, nala_pkgs)
	get_changes(cache, nala_pkgs, "remove")


def _install(pkg_names: list[str] | None, ctx: typer.Context) -> None:
	sudo_check(pkg_names)
	if not pkg_names:
		if arguments.fix_broken:
			_fix_broken()
			return
		ctx.fail(_("{error} Missing packages to install").format(error=ERROR_PREFIX))

	pkg_names = dedupe_list(pkg_names)  # type: ignore[arg-type]
	check_state(cache := setup_cache(), nala_pkgs)
	not_exist = split_local(pkg_names, cache, nala_pkgs.local_debs)
	install_local(nala_pkgs, cache)

	pkg_names = cache.glob_filter(pkg_names)
	pkg_names = cache.virtual_filter(pkg_names)
	broken, not_found, ver_failed = check_broken(pkg_names, cache)
	not_found.extend(not_exist)

	if not_found or ver_failed:
		pkg_error(not_found, cache)

	nala_pkgs.user_explicit = [cache[pkg_name] for pkg_name in pkg_names]
	if not package_manager(pkg_names, cache):
		if not (error := BrokenError(cache, broken)).broken_install():
			error.unmarked_error(nala_pkgs.user_explicit)

	auto_remover(cache, nala_pkgs)
	get_changes(cache, nala_pkgs, "install")


def remove_completion(ctx: typer.Context) -> Generator[str, None, None]:
	"""Complete remove command arguments."""
	if not DPKG_STATE.exists():
		return
	regex = r"ok installed|half-installed|unpacked|half-configured"
	if "purge" in ctx.command_path:
		regex += r"|config-files"

	status = re.compile(regex)
	for package in DPKG_STATE.read_text(encoding="utf-8").split("\n\n"):
		pkg_name = pkg_status = None
		for line in package.splitlines():
			if len(feilds := line.split(": ")) == 1:
				continue
			if "Package" in feilds:
				pkg_name = feilds[1]
			if "Status" in feilds and status.findall(feilds[1]):
				pkg_status = feilds[1]
		if pkg_name and pkg_status:
			yield pkg_name


def package_completion(cur: str) -> Generator[str, None, None]:
	"""Complete install command arguments."""
	yield from run(
		["apt-cache", "--no-generate", "pkgnames", cur]
		if PKGCACHE.exists()
		else ["apt-cache", "pkgnames", cur],
		capture_output=True,
		check=True,
		text=True,
	).stdout.split()


@nala.command("update", help=_("Update package list."))
# pylint: disable=unused-argument
def _update(
	debug: bool = DEBUG,
	raw_dpkg: bool = RAW_DPKG,
	dpkg_option: List[str] = OPTION,
	verbose: bool = VERBOSE,
	man_help: bool = MAN_HELP,
) -> None:
	"""Update package list."""
	sudo_check()
	arguments.update = True
	setup_cache().print_upgradable()


@nala.command(help=_("Update package list and upgrade the system."))
# pylint: disable=unused-argument,too-many-arguments, too-many-locals
def upgrade(
	exclude: Optional[List[str]] = typer.Option(
		None,
		metavar="PKG",
		help=_("Specify packages to exclude during upgrade. Accepts glob*"),
	),
	purge: bool = PURGE,
	debug: bool = DEBUG,
	raw_dpkg: bool = RAW_DPKG,
	download_only: bool = DOWNLOAD_ONLY,
	remove_essential: bool = REMOVE_ESSENTIAL,
	full: bool = typer.Option(True, help=_("Toggle full-upgrade")),
	update: bool = UPDATE,
	auto_remove: bool = AUTO_REMOVE,
	install_recommends: bool = RECOMMENDS,
	install_suggests: bool = SUGGESTS,
	fix_broken: bool = FIX_BROKEN,
	assume_yes: bool = ASSUME_YES,
	dpkg_option: List[str] = OPTION,
	verbose: bool = VERBOSE,
	man_help: bool = MAN_HELP,
) -> None:
	"""Update package list and upgrade the system."""
	sudo_check()

	def _upgrade(
		full: bool = True,
		exclude: list[str] | None = None,
		nested_cache: Cache | None = None,
	) -> None:
		"""Upgrade pkg[s]."""
		cache = nested_cache or setup_cache()
		check_state(cache, nala_pkgs)

		is_upgrade = tuple(cache.upgradable_pkgs())
		protected = cache.protect_upgrade_pkgs(exclude)
		try:
			cache.upgrade(dist_upgrade=full)
		except apt_pkg.Error:
			if exclude:
				exclude = fix_excluded(protected, is_upgrade)
				if ask(_("Would you like us to protect these and try again?")):
					cache.clear()
					_upgrade(full, exclude, cache)
					sys.exit()
				sys.exit(
					_("{error} You have held broken packages").format(
						error=ERROR_PREFIX
					)
				)

			BrokenError(
				cache, tuple(pkg for pkg in cache if pkg.is_inst_broken)
			).broken_install()

		if kept_back := tuple(
			pkg
			for pkg in is_upgrade
			if not (pkg.marked_upgrade or pkg.marked_delete or pkg in protected)
		):
			BrokenError(cache, kept_back).held_pkgs(protected)
			check_term_ask()

		auto_remover(cache, nala_pkgs)
		get_changes(cache, nala_pkgs, "upgrade")

	_upgrade(full, exclude)


@nala.command(help=_("Install packages."))
# pylint: disable=unused-argument,too-many-arguments,too-many-locals
def install(
	ctx: typer.Context,
	pkg_names: Optional[List[str]] = typer.Argument(
		None,
		metavar="PKGS ...",
		help=_("Package(s) to install"),
		autocompletion=package_completion,
	),
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
	dpkg_option: List[str] = OPTION,
	verbose: bool = VERBOSE,
	man_help: bool = MAN_HELP,
) -> None:
	"""Install packages."""
	_install(pkg_names, ctx)


@nala.command(help=_("Remove packages."))
@nala.command("purge", help=_("Purge packages."))
@nala.command("uninstall", hidden=True)
# pylint: disable=unused-argument,too-many-arguments
def remove(
	pkg_names: List[str] = typer.Argument(
		...,
		metavar="PKGS ...",
		help=_("Package(s) to remove/purge"),
		autocompletion=remove_completion,
	),
	purge: bool = PURGE,
	debug: bool = DEBUG,
	raw_dpkg: bool = RAW_DPKG,
	download_only: bool = DOWNLOAD_ONLY,
	remove_essential: bool = REMOVE_ESSENTIAL,
	update: bool = UPDATE,
	auto_remove: bool = AUTO_REMOVE,
	fix_broken: bool = FIX_BROKEN,
	assume_yes: bool = ASSUME_YES,
	dpkg_option: List[str] = OPTION,
	verbose: bool = VERBOSE,
	man_help: bool = MAN_HELP,
) -> None:
	"""Remove or Purge packages."""
	command_help("uninstall", "remove", update)
	_remove(pkg_names)


@nala.command("autoremove", help=_("Autoremove packages that are no longer needed."))
@nala.command("autopurge", help=_("Autopurge packages that are no longer needed."))
# pylint: disable=unused-argument,too-many-arguments
def _auto_remove(
	purge: bool = PURGE,
	debug: bool = DEBUG,
	config: bool = CONFIG,
	raw_dpkg: bool = RAW_DPKG,
	download_only: bool = DOWNLOAD_ONLY,
	remove_essential: bool = REMOVE_ESSENTIAL,
	update: bool = UPDATE,
	fix_broken: bool = FIX_BROKEN,
	assume_yes: bool = ASSUME_YES,
	dpkg_option: List[str] = OPTION,
	verbose: bool = VERBOSE,
	man_help: bool = MAN_HELP,
) -> None:
	"""Command for autoremove."""
	sudo_check()
	if config and not arguments.is_purge():
		sys.exit(
			_(
				"{error} {config} must be used with either {autoremove} or {autopurge}."
			).format(
				error=ERROR_PREFIX,
				config=color("--config", "YELLOW"),
				autoremove=color("autoremove --purge", "YELLOW"),
				autopurge=color("autopurge", "YELLOW"),
			)
		)
	cache = setup_cache()
	check_state(cache, nala_pkgs)
	auto_remover(cache, nala_pkgs, config)
	get_changes(cache, nala_pkgs, "remove")


@nala.command(help=_("Show package details."))
@nala.command("info", hidden=True)
# pylint: disable=unused-argument
def show(
	pkg_names: List[str] = typer.Argument(
		...,
		help=_("Package(s) to show"),
		autocompletion=package_completion,
	),
	debug: bool = DEBUG,
	verbose: bool = VERBOSE,
	all_versions: bool = ALL_VERSIONS,
	man_help: bool = MAN_HELP,
) -> None:
	"""Show package details."""
	command_help("info", "show", None)
	cache = Cache()
	not_found: list[str] = []
	pkg_names = cache.glob_filter(pkg_names, show=True)
	pkg_names = cache.virtual_filter(pkg_names)
	additional_records = 0
	for num, pkg_name in enumerate(pkg_names):
		if pkg_name in cache:
			pkg = cache[pkg_name]
			additional_records += show_main(num, pkg)
			continue
		pkg_not_found(pkg_name, cache, not_found)

	if additional_records and not arguments.all_versions:
		additional_notice(additional_records)

	if not_found:
		for error in not_found:
			eprint(error)
		sys.exit(1)


@nala.command(help=_("Search package names and descriptions."))
# pylint: disable=unused-argument,too-many-arguments,too-many-locals
def search(
	word: str = typer.Argument(
		...,
		help=_("Regex or word to search for"),
		autocompletion=package_completion,
	),
	debug: bool = DEBUG,
	full: bool = FULL,
	names: bool = NAMES,
	installed: bool = INSTALLED,
	nala_installed: bool = NALA_INSTALLED,
	upgradable: bool = UPGRADABLE,
	upgradeable: bool = UPGRADEABLE,
	all_versions: bool = ALL_VERSIONS,
	all_arches: bool = ALL_ARCHES,
	virtual: bool = VIRTUAL,
	verbose: bool = VERBOSE,
	man_help: bool = MAN_HELP,
) -> None:
	"""Search package names and descriptions."""
	cache = Cache()
	found: list[tuple[Package, Version]] = []
	user_installed = (
		get_list(get_history("Nala"), "User-Installed") if nala_installed else []
	)

	pattern = (
		compile_regex(fnmatch.translate(word[2:]))
		if word.startswith("g/")
		else compile_regex(word)
	)

	arches = apt_pkg.get_architectures()

	for pkg in cache:
		if nala_installed and pkg.name not in user_installed:
			continue
		if arguments.installed and not pkg.installed:
			continue
		if arguments.upgradable and not pkg.is_upgradable:
			continue
		if arguments.virtual and not cache.is_virtual_package(pkg.name):
			continue
		if arguments.all_arches or pkg.architecture() in arches:
			found.extend(search_name(pkg, pattern))

	if not found:
		sys.exit(_("{error} {regex} not found.").format(error=ERROR_PREFIX, regex=word))
	iter_search(found)


@nala.command("list", help=_("List packages based on package names."))
# pylint: disable=unused-argument,too-many-arguments,too-many-locals
def list_pkgs(
	pkg_names: Optional[List[str]] = typer.Argument(
		None,
		help=_("Package(s) to list."),
		autocompletion=package_completion,
	),
	debug: bool = DEBUG,
	full: bool = FULL,
	installed: bool = INSTALLED,
	nala_installed: bool = NALA_INSTALLED,
	upgradable: bool = UPGRADABLE,
	upgradeable: bool = UPGRADEABLE,
	all_versions: bool = ALL_VERSIONS,
	virtual: bool = VIRTUAL,
	verbose: bool = VERBOSE,
	man_help: bool = MAN_HELP,
) -> None:
	"""List packages based on package names."""
	cache = Cache()
	user_installed = (
		get_list(get_history("Nala"), "User-Installed") if nala_installed else []
	)

	patterns: list[Pattern[str]] = []
	if pkg_names:
		for name in pkg_names:
			if name.startswith("r/"):
				# Take out the prefix when we compile the regex
				patterns.append(compile_regex(name[2:]))
				continue
			# Otherwise we default to glob only for list
			patterns.append(compile_regex(fnmatch.translate(name)))

	def _list_gen() -> Generator[
		tuple[Package, Version | tuple[Version, ...]], None, None
	]:
		"""Generate to speed things up."""
		for pkg in cache:
			if nala_installed and pkg.name not in user_installed:
				continue
			if arguments.installed and not pkg.installed:
				continue
			if arguments.upgradable and not pkg.is_upgradable:
				continue
			if arguments.virtual and not cache.is_virtual_package(pkg.name):
				continue
			if pkg_names:
				for regex in patterns:
					# Match against the shortname so that arch isn't included
					if regex.search(pkg.shortname):
						yield (pkg, get_version(pkg, inst_first=True))
						continue
				# If names were supplied and no matches
				# we don't want to grab everything
				continue

			# In this case no names were supplied so we list everything
			yield (pkg, get_version(pkg, inst_first=True))

	if not iter_search(_list_gen()):
		sys.exit(_("Nothing was found to list."))


@nala.command(help=_("Clear out the local archive of downloaded package files."))
# pylint: disable=unused-argument
def clean(
	lists: bool = LISTS,
	fetch: bool = FETCH,
	debug: bool = DEBUG,
	verbose: bool = VERBOSE,
	man_help: bool = MAN_HELP,
) -> None:
	"""Clear out the local archive of downloaded package files."""
	sudo_check()
	if lists:
		iter_remove(LISTS_DIR)
		iter_remove(LISTS_PARTIAL_DIR)
		print(_("Package lists have been cleaned"))
		return
	if fetch:
		NALA_SOURCES.unlink(missing_ok=True)
		print(_("Nala sources.list has been cleaned"))
		return
	iter_remove(ARCHIVE_DIR)
	iter_remove(PARTIAL_DIR)
	iter_remove(LISTS_PARTIAL_DIR)
	vprint(
		_("Removing {cache}\nRemoving {src_cache}").format(
			cache=PKGCACHE, src_cache=SRCPKGCACHE
		)
	)

	PKGCACHE.unlink(missing_ok=True)
	SRCPKGCACHE.unlink(missing_ok=True)
	print(_("Cache has been cleaned"))


@nala.command(hidden=True, help=_("I beg, pls moo"))
# pylint: disable=unused-argument
def moo(
	moos: Optional[List[str]] = typer.Argument(None, hidden=True),
	update: bool = typer.Option(None, hidden=True),
) -> None:
	"""I beg, pls moo."""
	print(CAT)
	can_no_moo = _("I can't moo for I'm a cat")
	print(f'{ELLIPSIS}"{can_no_moo}"{ELLIPSIS}')
	if update:
		what_did_you_expect = _("What did you expect to update?")
		print(f'{ELLIPSIS}"{what_did_you_expect}"{ELLIPSIS}')
		return
	if update is not None:
		what_did_you_expect = _("What did you expect no-update to do?")
		print(f'{ELLIPSIS}"{what_did_you_expect}"{ELLIPSIS}')
