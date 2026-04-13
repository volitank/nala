:title: nala-history

NAME
====

nala-history - subcommand for interacting with nala's history

SYNOPSIS
========

**nala history** [*ID*] [*--options*]

**nala history** {*undo* | *redo*} *ID* [*--options*]

DESCRIPTION
===========

**nala history** with no *ID* or subcommand shows a summary of stored package transactions.

**nala history** [*ID*] shows information about a specific transaction.

**nala history undo** [*ID*] replays the inverse of the stored package transaction.

**nala history redo** [*ID*] replays the stored package transaction again.

History entries are stored as numbered JSON files under */var/lib/nala/history*.

This command currently documents and implements list/detail plus *undo* and *redo* only.

OPTIONS
=======

--debug
	Print helpful information for solving issues.
	If you're submitting a bug report try running the command again with *--debug*
	and providing the output to the devs, it may be helpful.

-v, --verbose
	Disable scrolling text and print extra information

--tui
	Turn the TUI on if it is disabled in the config.

--no-tui
	Turn the TUI off. This takes precedence over other UI options.

-h, --help
	Shows this man page.

COPYRIGHT
=========

Copyright (C) 2021, 2022 Blake Lee
