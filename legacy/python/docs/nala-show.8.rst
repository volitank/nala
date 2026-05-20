:title: nala-show

NAME
====

nala-show - Show information about a package

SYNOPSIS
========

**nala show** [*--options*] PKGS ...

DESCRIPTION
===========

Show information about a package such as the name, version, dependencies etc.

This command is very similar to **apt show** except information is highlighted for readability.

OPTIONS
=======

**-a**, **--all-versions**
	Show all versions of a package.

**--debug**
	Print helpful information for solving issues.
	If you're submitting a bug report try running the command again with *--debug*
	and providing the output to the devs, it may be helpful.

**-v**, **--verbose**
	Disable scrolling text and print extra information

**-h**, **--help**
	Shows this man page.

COPYRIGHT
=========

Copyright (C) 2021, 2022 Blake Lee
