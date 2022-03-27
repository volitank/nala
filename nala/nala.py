#                 __
#    ____ _____  |  | _____
#   /    \\__  \ |  | \__  \
#  |   |  \/ __ \|  |__/ __ \_
#  |___|  (____  /____(____  /
#       \/     \/          \/
#
# Copyright (C) 2010 Tatsuhiro Tsujikawa
# Copyright (C) 2021, 2022 Blake Lee
#
# This file is part of nala
# nala is based upon apt-metalink https://github.com/tatsuhiro-t/apt-metalink
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
"""Main module for Nala which facilitates apt."""
from __future__ import annotations

import re
import sys

import apt_pkg
from apt.package import Package, Version

from nala import _, color
from nala.cache import Cache
from nala.constants import (
	ARCHIVE_DIR,
	CAT_ASCII,
	ERROR_PREFIX,
	LISTS_PARTIAL_DIR,
	PARTIAL_DIR,
	PKGCACHE,
	SRCPKGCACHE,
)
from nala.error import broken_error, broken_pkg, pkg_error, unmarked_error
from nala.history import history_clear, history_info, history_summary, history_undo
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
from nala.options import arguments
from nala.rich import search_progress
from nala.search import print_search, search_name
from nala.show import additional_notice, pkg_not_found, show_main
from nala.utils import (
	PackageHandler,
	ask,
	dprint,
	eprint,
	iter_remove,
	pkg_installed,
	sudo_check,
	vprint,
)

nala_pkgs = PackageHandler()


def upgrade(nested_cache: Cache | None = None) -> None:
	"""Upgrade pkg[s]."""
	cache = nested_cache or setup_cache()
	check_state(cache, nala_pkgs)

	is_upgrade = cache.upgradable_pkgs()
	protected = cache.protect_upgrade_pkgs()
	try:
		cache.upgrade(dist_upgrade=not arguments.no_full)
	except apt_pkg.Error as error:
		if arguments.exclude:
			arguments.exclude = fix_excluded(protected, is_upgrade)
			if ask(_("Would you like us to protect these and try again?")):
				cache.clear()
				upgrade(cache)
				sys.exit()
			sys.exit(
				_("{error} You have held broken packages").format(error=ERROR_PREFIX)
			)
		raise error from error

	if kept_back := [pkg for pkg in is_upgrade if not pkg.is_upgradable]:
		cache.clear()
		print(color(_("The following packages were kept back:"), "YELLOW"))
		for pkg in kept_back:
			broken_pkg(pkg, cache)
		check_term_ask()
		cache.upgrade(dist_upgrade=not arguments.no_full)

	auto_remover(cache, nala_pkgs)
	get_changes(cache, nala_pkgs, upgrade=True)


def install(pkg_names: list[str]) -> None:
	"""Install pkg[s]."""
	if arguments.fix_broken:
		fix_broken()
		return
	cache = setup_cache()
	check_state(cache, nala_pkgs)

	not_exist = split_local(pkg_names, cache, nala_pkgs.local_debs)
	install_local(nala_pkgs, cache)

	pkg_names = cache.glob_filter(pkg_names)
	pkg_names = cache.virtual_filter(pkg_names)
	broken, not_found, ver_failed = check_broken(pkg_names, cache)
	not_found.extend(not_exist)

	if not_found or ver_failed:
		pkg_error(not_found, cache, terminate=True)

	pkgs = [cache[pkg_name] for pkg_name in pkg_names]
	if (
		not package_manager(pkg_names, cache)
		# We also check to make sure that all the packages are still
		# Marked upgrade or install after the package manager is run
		or not all(
			(pkg.marked_upgrade or pkg.marked_install or pkg.marked_downgrade)
			for pkg in pkgs
		)
	) and not broken_error(broken, cache):
		unmarked_error(pkgs)

	auto_remover(cache, nala_pkgs)
	get_changes(cache, nala_pkgs)


def remove(pkg_names: list[str]) -> None:
	"""Remove or Purge pkg[s]."""
	cache = setup_cache()
	check_state(cache, nala_pkgs)

	_purge = arguments.command == "purge"
	pkg_names = cache.glob_filter(pkg_names)
	pkg_names = cache.virtual_filter(pkg_names)
	broken, not_found, ver_failed = check_broken(
		pkg_names, cache, remove=True, purge=_purge
	)

	if not_found or ver_failed:
		pkg_error(not_found, cache, remove=True)

	if not package_manager(pkg_names, cache, remove=True, purge=_purge):
		broken_error(
			broken,
			cache,
			tuple(
				pkg
				for pkg in cache
				if pkg.is_installed and pkg_installed(pkg).dependencies
			),
		)

	auto_remover(cache, nala_pkgs, _purge)
	get_changes(cache, nala_pkgs, remove=True)


def auto_remove() -> None:
	"""Command for autoremove."""
	cache = setup_cache()
	_purge = arguments.command == "autopurge"
	check_state(cache, nala_pkgs)
	auto_remover(cache, nala_pkgs, _purge)
	get_changes(cache, nala_pkgs, remove=True)


def fix_broken(nested_cache: Cache | None = None) -> None:
	"""Attempt to fix broken packages, if any."""
	cache = nested_cache or setup_cache()
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
	get_changes(cache, nala_pkgs)


def show(pkg_names: list[str]) -> None:
	"""Show package information."""
	cache = setup_cache()
	not_found: list[str] = []
	pkg_names = cache.glob_filter(pkg_names)
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


def search() -> None:
	"""Search command entry point."""
	if not (search_term := arguments.args):
		sys.exit(
			_("{error} you must specify a pattern to search").format(error=ERROR_PREFIX)
		)
	cache = setup_cache()
	found: list[tuple[Package, Version]] = []
	if search_term == "*":
		search_term = ".*"
	try:
		search_pattern = re.compile(search_term, re.IGNORECASE)
	except re.error as error:
		sys.exit(
			_(
				"{error} failed regex compilation '{error_msg} at position {position}"
			).format(error=ERROR_PREFIX, error_msg=error.msg, position=error.pos)
		)
	with search_progress as progress:
		task = progress.add_task("", total=len(cache))
		arches = apt_pkg.get_architectures()
		with cache.actiongroup():  # type: ignore[attr-defined]
			for pkg in cache:
				progress.advance(task)
				if arguments.installed and not pkg.installed:
					continue
				if pkg.architecture() in arches:
					search_name(pkg, search_pattern, found)
	if not found:
		sys.exit(
			_("{error} {regex} not found.").format(
				error=ERROR_PREFIX, regex=search_term
			)
		)
	print_search(found)


def history() -> None:
	"""Coordinate the history command."""
	mode = arguments.mode
	# Eventually we should probably make argparser better and handle this for us.
	if mode and mode not in ("undo", "redo", "info", "clear"):
		sys.exit(
			_("{error} {command} isn't a valid history command").format(
				error=ERROR_PREFIX, command=mode
			)
		)
	if mode and not arguments.id:
		sys.exit(_("{error} We need a transaction ID").format(error=ERROR_PREFIX))
	if mode in ("undo", "redo", "info"):
		try:
			# We are basically just type checking here
			int(arguments.id)
		except ValueError:
			sys.exit(_("{error} ID must be a number").format(error=ERROR_PREFIX))
	if mode in ("undo", "redo"):
		if mode == "undo":
			sudo_check(_("Nala needs root to undo history"))
		elif mode == "redo":
			sudo_check(_("Nala needs root to redo history"))
		history_undo(arguments.id, redo=mode == "redo")
		return

	if mode == "info":
		history_info(arguments.id)
		return

	if mode == "clear":
		sudo_check(_("Nala needs root to clear history"))
		history_clear(arguments.id)
		return
	history_summary()


def clean() -> None:
	"""Find and delete cache files."""
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


def moo() -> None:
	"""I beg, pls moo."""
	moos = arguments.moo
	moos = moos.count("moo")
	dprint(f"moo number is {moos}")
	if moos == 1:
		print(CAT_ASCII["2"])
	elif moos == 2:
		print(CAT_ASCII["3"])
	else:
		print(CAT_ASCII["1"])
	can_no_moo = _("I can't moo for I'm a cat")
	print(f'..."{can_no_moo}"...')
	if arguments.no_update:
		what_did_you_expect = _("What did you expect no-update to do?")
		print(f"...{what_did_you_expect}...")
	if arguments.update:
		what_did_you_expect = _("What did you expect to update?")
		print(f"...{what_did_you_expect}...")
