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
"""Nala argument parsing tests."""
from __future__ import annotations

import sys

import pytest
from constants import (ARGS, GLOBAL_SWITCHES,
				INTERACTIVE_SWITCHES, PACKAGE_COMMANDS, STORE_FALSE)

from nala.__main__ import main
from nala.options import parser


@pytest.mark.skip(reason="this function is called from within other tests")
def switch_test(command: str, switch: str, *args: str) -> None:
	"""Test switches in a multi-use function."""
	key = switch[2:].replace('-', '_')
	parser_args = [command, switch]

	if args:
		parser_args.extend(args)
	arguments = parser.parse_args(parser_args)
	assert arguments.command == command

	if switch in STORE_FALSE:
		assert not getattr(arguments, key)
	else:
		assert getattr(arguments, key)

	if command in PACKAGE_COMMANDS:
		assert arguments.args == ARGS

def test_with_no_arguments_passed() -> None:
	"""Test if Nala errors out as expected when no arguments are passed."""
	for command in PACKAGE_COMMANDS:
		try:
			sys.argv = ['nala', command]
			main()
		except SystemExit as error:
			assert error.code == 1

def test_with_unknown_arguments() -> None:
	"""Test if Nala errors out as expected when wrong arguments are passed."""
	try:
		sys.argv = ['nala', "lucien", "isn't", "command"]
		main()
	except SystemExit as error:
		assert error.code == 1

def test_with_no_packages_passed() -> None:
	"""Test behavior with no packages passed to the command.

	Tests install, remove and purge
	"""
	for command in PACKAGE_COMMANDS+('show',):
		try:
			sys.argv = [command]
			main()
		except SystemExit as error:
			assert error.code == 1

def test_with_unknown_flags() -> None:
	"""Test behavior with unknown flags passed to the command."""
	for command in PACKAGE_COMMANDS:
		try:
			sys.argv = [command, "--no-flag"]
			main()
		except SystemExit as error:
			assert error.code == 1

def test_all_package_switches() -> None:
	"""Test behavior with each install, remove and purge flag enabled."""
	for command in PACKAGE_COMMANDS:
		for switch in GLOBAL_SWITCHES+INTERACTIVE_SWITCHES:
			switch_test(command, switch, *ARGS)

def test_all_upgrade_switches() -> None:
	"""Test all update and upgrade switches."""
	for command in ('update', 'upgrade'):
		for switch in GLOBAL_SWITCHES+INTERACTIVE_SWITCHES+('--no-full',):
			switch_test(command, switch)

def test_all_show_switches() -> None:
	"""Test all show switches."""
	for switch in GLOBAL_SWITCHES+INTERACTIVE_SWITCHES+('--all-versions',):
		switch_test('show', switch, *ARGS)

def test_all_fetch_switches() -> None:
	"""Test all fetch switches."""
	arguments = parser.parse_args(['fetch'])
	assert arguments.fetches == 3
	assert arguments.debian is None
	assert arguments.ubuntu is None
	assert arguments.country is None
	assert not arguments.foss

	arguments = parser.parse_args([
		'fetch',
		'--fetches', '10',
		'--debian', 'sid',
		'--ubuntu', 'focal',
		'--country', 'US',
		'--foss'
	])
	assert arguments.fetches == 10
	assert arguments.debian == 'sid'
	assert arguments.ubuntu == 'focal'
	assert arguments.country == 'US'
	assert arguments.foss

def test_all_history_switches() -> None:
	"""Test all history switches."""
	for switch in GLOBAL_SWITCHES+INTERACTIVE_SWITCHES:
		switch_test('history', switch)

def test_history_bad_args() -> None:
	"""Test if the history errors when not given an integer type for <ID>."""
	for sub_command in ('info', 'undo', 'redo', 'clear'):
		try:
			sys.argv = ['history', sub_command, 'notanum']
			main()
		except SystemExit as error:
			assert error.code == 1

def test_all_clean_switches() -> None:
	"""Test all clean switches."""
	for switch in GLOBAL_SWITCHES:
		switch_test('history', switch)

def test_all_sudo_commands() -> None:
	"""Test if sudo commands fail without sudo."""
	superuser = ('update', 'upgrade', 'install', 'remove', 'fetch', 'clean')
	for command in superuser:
		try:
			sys.argv = [command]
			main()
		except SystemExit as error:
			assert error.code == 1

	for subcommand in ('undo', 'redo', 'clear'):
		try:
			sys.argv = ['history', subcommand, '1']
			main()
		except SystemExit as error:
			assert error.code == 1
