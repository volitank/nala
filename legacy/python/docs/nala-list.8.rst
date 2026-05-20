:title: nala-list

NAME
====

nala-list - List all packages or only packages based on the provided name

SYNOPSIS
========

**nala list** [*--options*] PKGS ...

DESCRIPTION
===========

List all packages or only packages based on the provided name, glob or regex.

By default **list** will only glob.

If you start your query with *r/*, **list** will only regex.

*Note: You may need to quote the pattern so the shell doesn't glob first*

Examples:

	Glob that will match **nala** and anything text after it:

		**sudo nala list** *"g/nala\*"*

	A question mark in a glob will match any single character:

		**sudo nala list** *"g/na?a"*

		This would match **nala** or even **nada**

Regex works similar to globbing but is much more powerful.

For more information on Glob and Regex pattern matching:

	**man** *glob.7*

	**man** *regex.7*

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
