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

from nala.constants import (ARCHIVE_DIR, CAT_ASCII, ERROR_PREFIX,
				LISTS_PARTIAL_DIR, PARTIAL_DIR, PKGCACHE, SRCPKGCACHE)
from nala.error import broken_error, pkg_error
from nala.history import (history_clear,
				history_info, history_summary, history_undo)
from nala.install import (auto_remover, check_broken, check_term_ask,
				get_changes, get_extra_pkgs, install_local, installed_found_deps,
				installed_missing_dep, package_manager, setup_cache, split_local)
from nala.options import arguments
from nala.rich import search_progress
from nala.search import print_search, search_name
from nala.show import additional_notice, check_virtual, show_main
from nala.utils import (PackageHandler, color, dprint,
				glob_filter, iter_remove, pkg_installed, sudo_check)

nala_pkgs = PackageHandler()

def upgrade() -> None:
	"""Upgrade pkg[s]."""
	cache = setup_cache()
	is_upgrade = [pkg for pkg in cache if pkg.is_upgradable]
	cache.upgrade(dist_upgrade=arguments.no_full)

	if kept_back := [pkg for pkg in is_upgrade if not pkg.is_upgradable]:
		for pkg in kept_back:
			print(f"{color(pkg.name, 'YELLOW')} was kept back")
		check_term_ask()

	auto_remover(cache, nala_pkgs)
	get_changes(cache, nala_pkgs, upgrade=True)

def install(pkg_names: list[str]) -> None:
	"""Install pkg[s]."""
	cache = setup_cache()
	dprint(f"Install pkg_names: {pkg_names}")
	not_exist = split_local(pkg_names, cache, nala_pkgs.local_debs)
	install_local(nala_pkgs)

	pkg_names = glob_filter(pkg_names, cache.keys())
	broken, not_found = check_broken(pkg_names, cache)
	not_found.extend(not_exist)

	if not_found:
		pkg_error(not_found, 'not found', terminate=True)

	if arguments.no_install_recommends:
		get_extra_pkgs('Recommends', pkg_names, cache, nala_pkgs.recommend_pkgs)
	if not arguments.install_suggests:
		get_extra_pkgs('Suggests', pkg_names, cache, nala_pkgs.suggest_pkgs)

	if not package_manager(pkg_names, cache):
		broken_error(broken)

	auto_remover(cache, nala_pkgs)
	get_changes(cache, nala_pkgs)

def remove(pkg_names: list[str]) -> None:
	"""Remove or Purge pkg[s]."""
	cache = setup_cache()
	_purge = arguments.command == 'purge'
	dprint(f"Remove pkg_names: {pkg_names}")
	not_found: list[str] = []

	pkg_names = glob_filter(pkg_names, cache.keys())
	broken, not_found = check_broken(
		pkg_names, cache, remove=True, purge=_purge
	)

	if not_found:
		pkg_error(not_found, 'not found')

	if not package_manager(
		pkg_names, cache,
		remove=True, deleted=nala_pkgs.deleted, purge=_purge):

		broken_error(
			broken,
			tuple(pkg for pkg in cache if pkg.is_installed and pkg_installed(pkg).dependencies)
		)

	dprint(f"Marked delete: {nala_pkgs.deleted}")

	auto_remover(cache, nala_pkgs, _purge)
	get_changes(cache, nala_pkgs, remove=True)

def purge(pkg_names: list[str]) -> None:
	"""Wrap the remove command as purge."""
	remove(pkg_names)

def fix_broken() -> None:
	"""Attempt to fix broken packages, if any."""
	cache = setup_cache()
	broken: list[Package] = []
	fixable: list[Package] = []
	fixer = apt_pkg.ProblemResolver(cache._depcache)
	for pkg in cache:
		if pkg.is_now_broken:
			try:
				pkg.mark_install()
				fixable.append(pkg)
			except apt_pkg.Error as error:
				if 'broken packages' not in str(error):
					raise error from error
				broken.append(pkg)
				cache.clear()
				fixer.clear(pkg._pkg)
				fixer.resolve(True)

	for pkg in broken:
		installed_missing_dep(pkg)
	for pkg in fixable:
		installed_found_deps(pkg, cache)

	auto_remover(cache, nala_pkgs)
	get_changes(cache, nala_pkgs)

def show(pkg_names: list[str]) -> None:
	"""Show package information."""
	cache = setup_cache()
	dprint(f"Show pkg_names: {pkg_names}")
	not_found: list[str] = []
	additional_records = 0
	for num, pkg_name in enumerate(pkg_names):
		if pkg_name in cache:
			pkg = cache[pkg_name]
			additional_records += show_main(num, pkg)
			continue

		if check_virtual(pkg_name, cache):
			continue
		not_found.append(f"{ERROR_PREFIX}{color(pkg_name, 'YELLOW')} not found")

	if additional_records and not arguments.all_versions:
		additional_notice(additional_records)

	if not_found:
		for error in not_found:
			print(error)
		sys.exit(1)

def search(search_term: str) -> None:
	"""Search command entry point."""
	if not search_term:
		sys.exit(f'{ERROR_PREFIX}You must specify a pattern to search')
	cache = setup_cache()
	found: list[tuple[Package, Version]] = []
	if search_term == '*':
		search_term = '.*'
	try:
		search_pattern = re.compile(search_term, re.IGNORECASE)
	except re.error as error:
		sys.exit(f"{ERROR_PREFIX}Failed Regex Compilation '{error.msg} at position {error.pos}'")
	with search_progress as progress:
		task = progress.add_task('Searching...', total=len(cache))
		arches = apt_pkg.get_architectures()
		for pkg in cache:
			if arguments.installed and not pkg.installed:
				progress.advance(task)
				continue
			if pkg.architecture() in arches:
				search_name(pkg, search_pattern, found)
			progress.advance(task)
	if not found:
		sys.exit(f"{ERROR_PREFIX}{color(search_term, 'YELLOW')} was not found.")
	print_search(found)

def history() -> None:
	"""Coordinate the history command."""
	mode = arguments.mode
	# Eventually we should probably make argparser better and handle this for us.
	if mode and mode not in ('undo', 'redo', 'info', 'clear'):
		sys.exit(f"{ERROR_PREFIX}'{mode}' isn't a valid history command")
	if mode and not arguments.id:
		sys.exit(f'{ERROR_PREFIX}We need a transaction ID...')
	if mode in ('undo', 'redo', 'info'):
		try:
			# We are basically just type checking here
			int(arguments.id)
		except ValueError:
			sys.exit(f'{ERROR_PREFIX}Option must be a number...')

	if mode in ('undo', 'redo'):
		sudo_check(f"{mode} history")
		history_undo(arguments.id, redo=mode == 'redo')
		return

	if mode == 'info':
		history_info(arguments.id)
		return

	if mode == 'clear':
		sudo_check('clear history')
		history_clear(arguments.id)
		return
	history_summary()

def clean() -> None:
	"""Find and delete cache files."""
	iter_remove(ARCHIVE_DIR, arguments.verbose)
	iter_remove(PARTIAL_DIR, arguments.verbose)
	iter_remove(LISTS_PARTIAL_DIR, arguments.verbose)
	if arguments.verbose:
		print(f'Removing {PKGCACHE}')
		print(f'Removing {SRCPKGCACHE}')
	elif arguments.debug:
		dprint(f'Removing {PKGCACHE}')
		dprint(f'Removing {SRCPKGCACHE}')
	PKGCACHE.unlink(missing_ok=True)
	SRCPKGCACHE.unlink(missing_ok=True)
	print("Cache has been cleaned")

def moo() -> None:
	"""I beg, pls moo."""
	moos = arguments.moo
	moos = moos.count('moo')
	dprint(f"moo number is {moos}")
	if moos == 1:
		print(CAT_ASCII['2'])
	elif moos == 2:
		print(CAT_ASCII['3'])
	else:
		print(CAT_ASCII['1'])
	print('..."I can\'t moo for I\'m a cat"...')
	if arguments.no_update:
		print("...What did you expect no-update to do?...")
	if arguments.update:
		print("...What did you expect to update?...")
