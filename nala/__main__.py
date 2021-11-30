# nala is a wrapper for the apt package manager.
# Copyright (C) 2021 Volitank

# nala is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.

# nala is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.tures

# You should have received a copy of the GNU General Public License
# along with nala.  If not, see <https://www.gnu.org/licenses/>.

## Special thanks to Max Taggart for Columnar. nala uses a modified version
## https://github.com/MaxTaggart/columnar
## Special thanks to Tatsuhiro Tsujikawa for apt-metalink. nala uses a modified version
## https://github.com/tatsuhiro-t/apt-metalink

from sys import argv
from nala.utils import LION_ASCII, LION_ASCII2, DEBUG, esyslog, CAT_ASCII
from nala.fetch import fetch
from nala.utils import logger, dprint, nodate_format
from nala.options import arg_parse
from nala.nala import nala
import logging
from os import geteuid
from nala import __version__
from getpass import getuser

def _main():
	parser = arg_parse()
	arguments = parser.parse_args()
	command = arguments.command
	debug = arguments.debug
	no_update = arguments.no_update
	assume_yes = arguments.assume_yes
	download_only = arguments.download_only

	su = geteuid()
	if debug:
		std_err_handler = logging.StreamHandler()
		std_err_handler.setFormatter(nodate_format)
		logger.addHandler(std_err_handler)
		logger.setLevel(DEBUG)

	if not command:
		parser.print_help()
		exit()

	dprint(f"Argparser = {arguments}")

	superuser= ['update', 'upgrade', 'install', 'remove', 'fetch']
	require_update = ['update', 'upgrade', 'install']
	no_update_list = ['remove', 'show', 'history']
	apt_init = ['update', 'upgrade', 'install', 'remove', 'show', 'history']

	if command in superuser:
		if su != 0:
			esyslog(f"{getuser()} tried to run [{' '.join(com for com in argv)}] without permission")
			exit(f"nala needs root to {command}")

	if command in apt_init:
		if command in no_update_list:
			no_update = True

		apt = nala(
			download_only=download_only,
			assume_yes=assume_yes,
			no_update=no_update,
			debug=debug
		)

	if command in ['update', 'upgrade']:
		apt.upgrade(dist_upgrade=arguments.no_full)

	if command == 'install':
		args = arguments.args
		if not args:
			print('You must specify a package to install')
			exit()
		apt.install(args)

	if command == 'remove':
		args = arguments.args
		apt.remove(args)

	if command == 'fetch':
		foss = arguments.foss
		fetches = arguments.fetches
		debian = arguments.debian
		ubuntu = arguments.ubuntu
		country = arguments.country

		fetch(fetches, foss, debian, ubuntu, country, assume_yes)

	if command == 'show':
		args = arguments.args
		if not args:
			print('You must specify a package to show')
			exit()
		apt.show(args)

	if command == 'history':
		id = arguments.id
		mode = arguments.mode
		if mode == 'undo':
			if id:
				apt.history_undo(id)
			else:
				print('\nWe need a transaction ID')
		elif mode == 'redo':
			if id:
				apt.history_undo(id, redo=True)
			else:
				print('\nWe need a transaction ID')
		elif mode == 'info':
			if id:
				apt.history_info(id)
			else:
				print('\nWe need a transaction ID')
		elif mode == 'clear':
			if su != 0:
				esyslog(f"{getuser()} tried to run [{' '.join(com for com in argv)}] without permission")
				exit(f"nala needs root to {command}")
			if id:
				apt.history_clear(id)
			else:
				print('\nWe need a transaction ID')
		else:
			apt.history()
	
	if command == 'moo':
		moos = arguments.moo
		moos = moos.count('moo')
		dprint(f"moo number is {moos}")
		if moos == 1:
			print(LION_ASCII)
		elif moos == 2:
			print(LION_ASCII2)
		else:
			print(CAT_ASCII)
		print('..."I can\'t moo for I\'m a cat"...')
		if no_update:
			print("...What did you expect --no-update to do?...")

def main():
	try:
		_main()
	except KeyboardInterrupt:
		print('\nExiting at your request')
		exit()

