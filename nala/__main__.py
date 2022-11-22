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
"""The main module for Nala."""
from __future__ import annotations

import errno
import sys

# Imports to get all of the subcommands into typer
import nala.fetch as _fetch  # pylint: disable=unused-import
import nala.nala as _nala  # pylint: disable=unused-import
from nala import _
from nala.constants import ERROR_PREFIX
from nala.options import nala
from nala.utils import eprint


def main() -> None:
	"""Nala function to reference from the entry point."""
	try:
		nala()
	except KeyboardInterrupt:
		eprint("\n" + _("Exiting at your request."))
		sys.exit(130)
	except BrokenPipeError:
		sys.stderr.close()
	except OSError as error:
		if error.errno == errno.ENOSPC:
			sys.exit(_("{error} No space left on device.").format(error=ERROR_PREFIX))
		raise error from error
