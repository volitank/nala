:title: nala-list

NAME
====

nala-list - List all packages or only packages based on the provided name

SYNOPSIS
========

**nala list** [*--options*] PKGS ...

DESCRIPTION
===========

List all packages or only packages based on the provided name

The package name can be a regex or glob

This command is very similar to **apt list**

OPTIONS
=======

**--full**
	Print the full description of each package.

**-i**, **--installed**
	List only packages that are installed.

**-N**, **--nala-installed**
	List only packages explicitly installed with **nala**.

**-u**, **--upgradable**
	List only packages that are upgradable.

**-a**, **--all-versions**
	Show all versions of a package.

**-V**, **--virtual**
	List only virtual packages.

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
