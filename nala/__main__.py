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

import errno
import sys

from nala import _
from nala.constants import ARCHIVE_DIR, ERROR_PREFIX, NALA_LOGDIR
from nala.fetch import fetch
from nala.install import setup_cache
from nala.nala import (
	auto_remove,
	clean,
	history,
	install,
	moo,
	remove,
	search,
	show,
	upgrade,
)
from nala.options import arguments
from nala.utils import arg_check, dprint, eprint, sudo_check, term

if str(ARCHIVE_DIR) == "/":
	sys.exit(
		_("{error} archive dir is '/'. This is dangerous and unsupported.").format(
			error=ERROR_PREFIX
		)
	)


def start_nala() -> bool:
	"""Start the nala command."""
	if arguments.command == "install":
		install(arguments.args)
		sys.exit()
	if arguments.command == "show":
		show(arguments.args)
		sys.exit()
	if arguments.command in ("remove", "purge"):
		remove(arguments.args)
		sys.exit()
	if arguments.command == "update":
		setup_cache().print_upgradeable()
		sys.exit()
	if arguments.command == "upgrade":
		upgrade()
		sys.exit()
	if arguments.command == "clean":
		clean()
		sys.exit()
	if arguments.command == "fetch":
		fetch()
		sys.exit()
	if arguments.command == "history":
		history()
		sys.exit()
	if arguments.command == "search":
		search()
		sys.exit()
	if arguments.command == "moo":
		moo()
		sys.exit()
	if arguments.command in ("autoremove", "autopurge"):
		auto_remove()
		sys.exit()
	return False


def _main() -> None:
	"""Nala Main."""
	arg_check()

	if term.is_su() and not NALA_LOGDIR.exists():
		NALA_LOGDIR.mkdir()

	kwarg = "\n    ".join(
		(f"{kwarg[0]} = {kwarg[1]},") for kwarg in arguments._get_kwargs()
	)
	dprint(f"Argparser = [\n    {kwarg}\n]")
	if arguments.command in (
		"update",
		"upgrade",
		"install",
		"remove",
		"fetch",
		"clean",
		"purge",
		"autoremove",
		"autopurge",
	):
		sudo_check(_("Nala needs root to {command}").format(command=arguments.command))

	if not start_nala():
		sys.exit(
			_("{error} Unknown error in 'apt_command' function").format(
				error=ERROR_PREFIX
			)
		)


def main() -> None:
	"""Nala function to reference from the entry point."""
	try:
		_main()
	except KeyboardInterrupt:
		eprint(_("\nExiting at your request"))
		sys.exit(130)
	except BrokenPipeError:
		sys.stderr.close()
	except OSError as error:
		if error.errno == errno.ENOSPC:
			sys.exit(_("{error} No space left on device").format(error=ERROR_PREFIX))
		raise error from error
