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

from pathlib import Path

from pyshell import pyshell

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
# Update these logs to go into nala logdir
DPKG_LOG = NALA_LOGDIR / 'dpkg-debug.log'
"""/var/log/nala/dpkg-debug.log"""
DPKG_STATUS_LOG = NALA_LOGDIR / 'dpkg-status.log'
"""/var/log/nala/dpkg-status.log"""

shell = pyshell(capture_output=True, text=True, check=True)

# Click Style Colors
RED = {'fg':'red', 'bold':True}
YELLOW = {'fg':'yellow', 'bold':True}
GREEN = {'fg':'green', 'bold':True}
BLUE = {'fg':'blue', 'bold':True}
CYAN = {'fg':'cyan', 'bold':True}
MAGENTA = {'fg':'magenta', 'bold':True}

def ask(question, default_no=False):
	"""resp = input(f'{question}? [Y/n]

	Y returns True
	N returns False
	"""
	while True:
		resp = input(f'{question}? [Y/n] ')
		if resp in ('y', 'Y'):
			return True
		elif resp in ('n', 'N'):
			return False
		elif resp == '':
			return not default_no
		else:
			print("Not a valid choice kiddo")

# Constants
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
LION_ASCII = (
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
LION_ASCII2 = (
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

CAT_ASCII = (
r"""
   |\---/|
   | ,_, |
    \_`_/-..----.
 ___/ `   ' ,""+ \  sk
(__...'   __\    |`.___.';
  (_,...'(_,.`__)/'.....+
"""
)
