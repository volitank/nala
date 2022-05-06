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

import sys

# Before we import anything else we need to make sure that this is in our path
# Else apt_pkg will give an import error. Conda is an example of this breaking
# pylint: disable=wrong-import-position
PY_PATH = "/usr/lib/python3/dist-packages"
if PY_PATH not in sys.path:
	sys.path.append(PY_PATH)
import gettext

import apt_pkg
from rich.console import Console
from rich.traceback import install

install(show_locals=True)

if "APT" not in apt_pkg.config:
	apt_pkg.init_config()
apt_pkg.init_system()

__version__ = "0.8.2"

console = Console()

COLOR_CODES: dict[str, str | int] = {
	"RESET": "\x1b[0m",
	"ITALIC": "\x1b[3m",
	"RED": 31,
	"GREEN": 32,
	#'YELLOW' : 33,
	# We set Yellow as more of an orange for light terminals.
	"YELLOW": "38;5;202",
	"BLUE": 34,
	"MAGENTA": 35,
	"CYAN": 36,
	"WHITE": 37,
}


def color(text: object, text_color: str = "") -> str:
	"""Return bold text in the color of your choice."""
	if not console.is_terminal or console.is_dumb_terminal:
		return f"{text}"
	if not text_color:
		# Just return bolded text
		return f"\x1b[1m{text}{COLOR_CODES['RESET']}"
	# Return bolded choice of color
	return f"\x1b[1;{COLOR_CODES[text_color]}m{text}{COLOR_CODES['RESET']}"


def color_version(version: str) -> str:
	"""Color version number."""
	if not console.is_terminal or console.is_dumb_terminal:
		return f"({version})"
	return f"{color('(')}{color(version, 'BLUE')}{color(')')}"


translate = gettext.translation("nala", fallback=True)


def _(msg: str) -> str:
	"""Gettext translator."""
	return msg if console.options.ascii_only else translate.gettext(msg)
