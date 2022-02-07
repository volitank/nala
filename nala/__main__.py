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
"""The main module for Nala."""
from __future__ import annotations

import sys
from getpass import getuser
from typing import NoReturn

from nala.constants import (ARCHIVE_DIR, CAT_ASCII, ERROR_PREFIX,
				LISTS_PARTIAL_DIR, PARTIAL_DIR, PKGCACHE, SRCPKGCACHE)
from nala.fetch import fetch
from nala.history import history, history_clear, history_info, history_undo
from nala.logger import dprint, esyslog
from nala.nala import Nala
from nala.options import arguments, parser
from nala.utils import iter_remove, term

if str(ARCHIVE_DIR) == '/':
	sys.exit(ERROR_PREFIX+"archive dir is '/'. This is dangerous and unsupported.")

def _main() -> None:
	"""Nala Main."""
	if not arguments.command and not arguments.update:
		parser.print_help()
		sys.exit(1)

	dprint(f"Argparser = {arguments}")
	superuser= ('update', 'upgrade', 'install', 'remove', 'fetch', 'clean')
	apt_init = ('update', 'upgrade', 'install', 'remove', 'show', 'history', 'purge', None)

	if arguments.command in superuser:
		sudo_check(arguments.command)

	if arguments.command in apt_init:
		apt_command()
	else:
		not_apt_command()

def apt_command() -> NoReturn:
	"""Command which require initializing the apt cache."""
	apt = init_apt()
	if arguments.command in ('update', 'upgrade'):
		apt.upgrade(dist_upgrade=arguments.no_full)

	elif arguments.command == 'install':
		arg_check(arguments.args, 'install')
		apt.install(arguments.args)

	elif arguments.command in ('remove', 'purge'):
		purge = arguments.command == 'purge'
		args = arguments.args
		arg_check(arguments.args, arguments.command)
		apt.remove(args, purge=purge)

	elif arguments.command == 'show':
		arg_check(arguments.args, 'show')
		apt.show(arguments.args)

	elif arguments.command == 'history':
		nala_history(apt)

	elif not arguments.update:
		sys.exit(ERROR_PREFIX+'unknown error in "apt_command" function')
	sys.exit(0)

def not_apt_command() -> NoReturn:
	"""Command which does not require initializing the apt cache."""
	if arguments.command == 'clean':
		clean()
	elif arguments.command == 'fetch':
		fetch()
	elif arguments.command == 'moo':
		moo_pls()
	else:
		sys.exit(ERROR_PREFIX+'unknown error in "apt_command" function')
	sys.exit(0)

def arg_check(args: list[str], msg: str) -> None:
	"""Check arguments and errors if no packages are specified.

	If args exists then duplicates will be removed.
	"""
	if not args:
		sys.exit(ERROR_PREFIX+f'You must specify a package to {msg}')
	dedupe = []
	for arg in arguments.args:
		if arg not in dedupe:
			dedupe.append(arg)
	arguments.args = dedupe

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

def nala_history(apt: Nala) -> None:
	"""Coordinate the history command."""
	mode = arguments.mode
	# Eventually we should probably make argparser better and handle this for us.
	if mode and mode not in ('undo', 'redo', 'info', 'clear'):
		sys.exit(ERROR_PREFIX+f"'{mode}' isn't a valid history command")
	if mode and not arguments.id:
		sys.exit(ERROR_PREFIX+'We need a transaction ID...')
	if mode in ('undo', 'redo', 'info'):
		try:
			# We are basically just type checking here
			int(arguments.id)
		except ValueError:
			sys.exit(ERROR_PREFIX+'Option must be a number...')
	if not mode:
		history()
	if mode == 'undo':
		sudo_check('undo history')
		history_undo(apt, arguments.id)

	elif mode == 'redo':
		sudo_check('redo history')
		history_undo(apt, arguments.id, redo=True)

	elif mode == 'info':
		history_info(arguments.id)

	elif mode == 'clear':
		sudo_check('clear history')
		history_clear(arguments.id)

def sudo_check(root_action: str) -> None:
	"""Check for root and exits if not root."""
	if not term.is_su():
		esyslog(f'{getuser()} tried to run [{" ".join(sys.argv)}] without permission')
		sys.exit(ERROR_PREFIX+f'Nala needs root to {root_action}')

def init_apt() -> Nala:
	"""Initialize Nala and determines if we update the cache or not."""
	no_update_list = ('remove', 'show', 'history', 'install', 'purge')
	no_update = arguments.no_update
	if arguments.command in no_update_list:
		no_update = True
	if arguments.update:
		no_update = False

	return Nala(no_update)

def moo_pls() -> None:
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

def main() -> None:
	"""Nala function to reference from the entry point."""
	try:
		_main()
	except KeyboardInterrupt:
		print('\nExiting at your request')
		sys.exit(130)
