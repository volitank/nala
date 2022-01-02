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

from getpass import getuser
from os import geteuid
from sys import argv, exit

from nala.fetch import fetch
from nala.logger import dprint, esyslog
from nala.nala import nala
from nala.options import arguments, parser
from nala.utils import CAT_ASCII, LION_ASCII, LION_ASCII2, shell


def _main():
	command = arguments.command
	debug = arguments.debug
	no_update = arguments.no_update
	assume_yes = arguments.assume_yes
	download_only = arguments.download_only
	verbose = arguments.verbose
	update = arguments.update
	raw_dpkg = arguments.raw_dpkg

	su = geteuid()

	if not command and not update:
		parser.print_help()
		exit()

	dprint(f"Argparser = {arguments}")

	superuser= ('update', 'upgrade', 'install', 'remove', 'fetch', 'clean')
	apt_init = ('update', 'upgrade', 'install', 'remove', 'show', 'history', 'purge', None)

	if command in superuser and su != 0:
		esyslog(f'{getuser()} tried to run [{" ".join(argv)}] without permission')
		exit(f"Nala needs root to {command}")

	if command in apt_init:
		apt = init_apt(
			command, update,
			download_only, assume_yes,
			no_update, debug,
			verbose, raw_dpkg
		)

	if command in ('update', 'upgrade'):
		apt.upgrade(dist_upgrade=arguments.no_full)

	elif command == 'install':
		if not arguments.args:
			print('You must specify a package to install')
			exit()
		apt.install(arguments.args)

	elif command in ('remove', 'purge'):
		purge = command == 'purge'
		args = arguments.args
		apt.remove(args, purge=purge)

	elif command == 'fetch':
		fetch(
			arguments.fetches, arguments.foss,
			arguments.debian, arguments.ubuntu,
			arguments.country, assume_yes
		)

	elif command == 'show':
		if not arguments.args:
			print('You must specify a package to show')
			exit()
		apt.show(arguments.args)

	elif command == 'history':
		history(arguments, apt, su)

	elif command == 'clean':
		shell.apt.clean()
		print("Cache has been cleaned")

	elif command == 'moo':
		moo_pls(arguments, update, no_update)

def history(arguments, apt: nala, su):
	id = arguments.id
	mode = arguments.mode

	if mode and not id:
		print('We need a transaction ID..')
		exit(1)

	if mode in ('undo', 'redo', 'info'):
		try:
			id = int(id)
		except ValueError:
			print('Option must be a number..')
			exit(1)
	else:
		apt.history()
	if mode == 'undo':
		apt.history_undo(id)

	elif mode == 'redo':
		apt.history_undo(id, redo=True)

	elif mode == 'info':
		apt.history_info(id)

	elif mode == 'clear':
		if su != 0:
			esyslog(f'{getuser()} tried to run [{" ".join(argv)}] without permission')
			exit('Nala needs root to clear history')
		apt.history_clear(id)

def init_apt(
	command, update,
	download_only, assume_yes,
	no_update, debug,
	verbose, raw_dpkg):

	no_update_list = ('remove', 'show', 'history', 'install', 'purge')
	if command in no_update_list:
		no_update = True
	if update:
		no_update = False

	return nala(
		download_only=download_only,
		assume_yes=assume_yes,
		no_update=no_update,
		debug=debug,
		verbose=verbose,
		raw_dpkg=raw_dpkg
	)

def moo_pls(arguments, update, no_update):
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
		print("...What did you expect no-update to do?...")
	if update:
		print("...What did you expect to update?...")

def main():
	try:
		_main()
	except KeyboardInterrupt:
		print('\nExiting at your request')
		exit()
