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
"""Nala package manager."""
from __future__ import annotations

import gettext

import apt_pkg
from rich.console import Console

if "APT" not in apt_pkg.config:
	apt_pkg.init_config()
apt_pkg.init_system()

__version__ = '0.6.0'

console = Console()

COLOR_CODES: dict[str, str | int] = {
	'RESET' : '\x1b[0m',
	'ITALIC' : '\x1b[3m',
	'RED' : 31,
	'GREEN' : 32,
	'YELLOW' : 33,
	'BLUE' : 34,
	'MAGENTA' : 35,
	'CYAN' : 36,
	'WHITE' : 37,
}

def color(text: str, text_color: str = 'WHITE') -> str:
	"""Return bold text in the color of your choice."""
	if not console.is_terminal or console.is_dumb_terminal:
		return text
	return f"\x1b[1;{COLOR_CODES[text_color]}m{text}{COLOR_CODES['RESET']}"

def color_version(version: str) -> str:
	"""Color version number."""
	if not console.is_terminal or console.is_dumb_terminal:
		return f"({version})"
	return f"{color('(')}{color(version, 'BLUE')}{color(')')}"

translate = gettext.translation('nala', fallback=True)
def _(msg: str) -> str:
	"""Gettext translator."""
	return msg if console.options.ascii_only else translate.gettext(msg)
