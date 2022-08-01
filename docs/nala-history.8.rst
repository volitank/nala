:title: nala-history

NAME
====

nala-history - subcommand for interacting with nala's history

SYNOPSIS
========

**nala history** [*--options*]

**nala history** COMMAND [*--options*]

DESCRIPTION
===========

**nala history** with no subcommands will show a summary of all transactions made.

**nala history info** [*ID*] shows information about a specific transaction

	*last* can be used as the *ID* to get the last transaction

**nala history redo** [*ID*] redo the specified transaction

	*last* can be used as the *ID* to redo the last transaction

	This subcommand accepts the same switches as **install** or **remove**

**nala history undo** [*ID*] undo the specified transaction

	*last* can be used as the *ID* to undo the last transaction

	This subcommand accepts the same switches as **install** or **remove**

OPTIONS
=======

--installed
	Show only packages that were explicitly installed with Nala.

	For example, If you were to install *libreoffice* this switch will
	only show that package and not its dependencies.

--debug
	Print helpful information for solving issues.
	If you're submitting a bug report try running the command again with *--debug*
	and providing the output to the devs, it may be helpful.

-v, --verbose
	Disable scrolling text and print extra information

-h, --help
	Shows this man page.

COPYRIGHT
=========

Copyright (C) 2021, 2022 Blake Lee
