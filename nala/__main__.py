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

# pylint: disable=unused-import
from nala.constants import ARCHIVE_DIR, ERROR_PREFIX, NALA_LOGDIR
from nala.fetch import fetch
from nala.install import setup_cache
from nala.nala import (clean, fix_broken, history,
				install, moo, purge, remove, search, show, upgrade)
from nala.options import arguments, parser
from nala.utils import arg_check, dprint, eprint, sudo_check, term

if str(ARCHIVE_DIR) == '/':
	sys.exit(ERROR_PREFIX+"archive dir is '/'. This is dangerous and unsupported.")

def _main() -> None:
	"""Nala Main."""
	if arguments.raw_dpkg or not term.is_xterm():
		arguments.verbose = True
	if arguments.command == 'update':
		arguments.command = 'upgrade'

	if term.is_su() and not NALA_LOGDIR.exists():
		NALA_LOGDIR.mkdir()

	dprint(f"Argparser = {arguments}")
	if arguments.command in ('upgrade', 'install', 'remove', 'fetch', 'clean'):
		sudo_check(arguments.command)
	elif not arguments.command:
		if arguments.update:
			sudo_check('update package list')
			setup_cache()
			return
		if arguments.fix_broken:
			sudo_check('fix broken packages')
			fix_broken()
			return
		parser.print_help()
		sys.exit(1)

	arg_check()
	if arguments.command in ('install', 'show', 'search', 'remove', 'purge'):
		# eval should be safe here considering the commands are specifically defined.
		eval(f"{arguments.command}({arguments.args})") # pylint: disable=eval-used
		return
	if arguments.command in ('upgrade', 'clean', 'fetch', 'moo', 'history'):
		eval(f"{arguments.command}()") # pylint: disable=eval-used
		return
	sys.exit(ERROR_PREFIX+'unknown error in "apt_command" function')

def main() -> None:
	"""Nala function to reference from the entry point."""
	try:
		_main()
	except KeyboardInterrupt:
		eprint('\nExiting at your request')
		sys.exit(130)
