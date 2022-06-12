============
nala-history
============

----------------------------------------------
subcommand for interacting with nala's history
----------------------------------------------

:Date: 23 May 2022
:Copyright: Copyright (C) 2021, 2022 Blake Lee
:Version: 0.9.1
:Manual section: 8
:Manual group: NALA

SYNOPSIS
========

``nala history`` [`--options`]

``nala history`` COMMAND [`--options`]

DESCRIPTION
===========

``nala history`` with no subcommands will show a summary of all transactions made.

``nala history info`` [`ID`] whows information about a specific transaction

``nala history redo`` [`ID`] redo the specified transaction

	This subcommand accepts the same switches as ``install`` or ``remove``

``nala history undo`` [`ID`] undo the specified transaction

	This subcommand accepts the same switches as ``install`` or ``remove``

OPTIONS
=======

--installed
	Show only packages that were explicitly installed with Nala.

	For example, If you were to install `libreoffice` this switch will
	only show that package and not its dependencies.

--debug
	Print helpful information for solving issues.
	If you're submitting a bug report try running the command again with `--debug`
	and providing the output to the devs, it may be helpful.

-v, --verbose
	Disable scrolling text and print extra information

-h, --help
	Shows this man page.
