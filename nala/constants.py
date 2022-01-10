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
"""Module for file constants."""
from __future__ import annotations

from pathlib import Path
from shutil import get_terminal_size

import apt_pkg

# File Constants
LICENSE = Path('/usr/share/common-licenses/GPL-3')
"""/usr/share/common-licenses/GPL-3"""
NALA_SOURCES = Path('/etc/apt/sources.list.d/nala-sources.list')
"""/etc/apt/sources.list.d/nala-sources.list"""
NALA_LOGDIR = Path('/var/log/nala')
"""/var/log/nala"""
NALA_LOGFILE = NALA_LOGDIR / 'nala.log'
"""/var/log/nala/nala.log"""
NALA_DEBUGLOG = NALA_LOGDIR / 'nala-debug.log'
"""/var/log/nala/nala.debug.log"""
DPKG_LOG = NALA_LOGDIR / 'dpkg-debug.log'
"""/var/log/nala/dpkg-debug.log"""
DPKG_STATUS_LOG = NALA_LOGDIR / 'dpkg-status.log'
"""/var/log/nala/dpkg-status.log"""
NALA_HISTORY = Path('/var/lib/nala/history.json')
"""/var/lib/nala/history.json"""

# Apt Directories
ARCHIVE_DIR = Path(apt_pkg.config.find_dir('Dir::Cache::Archives'))
"""/var/cache/apt/archives/"""
PARTIAL_DIR = ARCHIVE_DIR / 'partial'
"""/var/cache/apt/archives/partial"""
LISTS_DIR = Path(apt_pkg.config.find_dir('Dir::State::Lists'))
"""/var/lib/apt/lists/"""
LISTS_PARTIAL_DIR = LISTS_DIR / 'partial'
"""/var/lib/apt/lists/partial"""
PKGCACHE = Path(apt_pkg.config.find_dir('Dir::Cache::pkgcache'))
"""/var/cache/apt/pkgcache.bin"""
SRCPKGCACHE = Path(apt_pkg.config.find_dir('Dir::Cache::srcpkgcache'))
"""/var/cache/apt/srcpkgcache.bin"""

# Term Constants
TERM_SIZE = get_terminal_size()
COLUMNS = TERM_SIZE.columns
LINES = TERM_SIZE.lines
ERROR_PREFIX = '\x1b[1;31mError: \x1b[0m'

COLOR_CODES: dict[str, str | int] = {
	'RESET' : '\x1b[0m',
	'RED' : 31,
	'GREEN' : 32,
	'YELLOW' : 33,
	'BLUE' : 34,
	'MAGENTA' : 35,
	'CYAN' : 36,
	'WHITE' : 37,
}

# dpkg constants
CONF_MESSAGE = (
	b"Configuration file '",
	b'==> Modified (by you or by a script) since installation.\r\n',
	b' ==> Package distributor has shipped an updated version.\r\n',
	b'   What would you like to do about it ?  Your options are:\r\n',
	b"    Y or I  : install the package maintainer's version\r\n",
	b'    N or O  : keep your currently-installed version\r\n',
	b'      D     : show the differences between the versions\r\n',
	b'      Z     : start a shell to examine the situation\r\n',
	b' The default action is to keep your current version.\r\n',
	b'*** config.inc.php (Y/I/N/O/D/Z) [default=N] ?',
)
CONF_ANSWER = (b'y', b'Y', b'i', b'I', b'n', b'N', b'o', b'O',)
NOTICES = (
	b'A reboot is required to replace the running dbus-daemon.',
	b'Please reboot the system when convenient.',
	b'The currently running kernel version is not the expected kernel version',
	b'so you should consider rebooting.',
	b'Please remove.',
	b'NOTICE:',
	b'Warning:'
)
SPAM = (
	# Stuff that's pretty useless
	'(Reading database', #'(Reading database ... 247588 files and directories currently installed.)'
	'files and directories currently installed.)',
	'Selecting previously unselected package', # 'Selecting previously unselected package chafa.'
	'Preparing to unpack', # 'Preparing to unpack .../2-chafa_1.8.0-1_amd64.deb ...'
	'Extracting templates from packages:',
	'Preconfiguring packages',
	'Reloading AppArmor profiles',
)
DPKG_STATUS = (
	b'Scanning processes...',
	b'Scanning candidates...',
	b'Scanning linux images...',
	b'Extracting templates from packages',
	b'Reading changelogs...'
)

# ASCII Art
LION_1 = (
r"""
         |\_
       -' | \
      /7     \
     /        `-_
     \-'_        `-.____________
      -- \                 /    `.
         /                 \      \
 _______/    /_       ______\      |__________-
(,__________/  `-.___(,_____________----------_)
"""
)
# I couldn't find an artist for these. If anyone knows let me know.
# I love to give credit when I can
LION_2 = (
r"""
    |\_
  -' | `.
 /7      `-._
/            `-.____________
\-'_                        `-._
 -- `-._                    |` -`.
       |\               \   |   `\\
       | \  \______...---\_  \    \\
       |  \  \           | \  |    ``-.__--.
       |  |\  \         / / | |       ``---'
     _/  /_/  /      __/ / _| |
    (,__/(,__/      (,__/ (,__/
"""
)

CAT_1 = (
r"""
   |\---/|
   | ,_, |
    \_`_/-..----.
 ___/ `   ' ,""+ \  sk
(__...'   __\    |`.___.';
  (_,...'(_,.`__)/'.....+
"""
)

# dicts to minimize imports
CAT_ASCII = {
	'1' : CAT_1,
	'2' : LION_1,
	'3' : LION_2
}

DPKG_MSG: dict[str, tuple[bytes, ...]] = {
	'CONF_MESSAGE' : CONF_MESSAGE,
	'CONF_ANSWER' : CONF_ANSWER,
	'NOTICES' : NOTICES,
	'DPKG_STATUS' : DPKG_STATUS,}
