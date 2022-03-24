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
"""The configuration module."""
from __future__ import annotations

import sys

from apt_pkg import Error, config, read_config_file

from nala import _, color
from nala.constants import ERROR_PREFIX, NOTICE_PREFIX
from nala.options import arguments
from nala.utils import eprint

CONF_FILE = '/etc/nala/nala.conf'

try:
	read_config_file(config, CONF_FILE)
except Error as error:
	eprint(
		str(error).replace('E:', f"{ERROR_PREFIX} ").replace(CONF_FILE, color(CONF_FILE, 'YELLOW'))
		)
	sys.exit(1)
except SystemError:
	eprint(_("{notice} Unable to read config file: {filename}. Using defaults").format(
		notice=NOTICE_PREFIX, filename = color(CONF_FILE, 'YELLOW')
	))

SCROLL = config.find_b('Nala::ScrollingText', True)
if arguments.verbose:
	SCROLL = False

AUTO_REMOVE = config.find_b('Nala::AutoRemove', True)
if arguments.no_autoremove:
	AUTO_REMOVE = False
