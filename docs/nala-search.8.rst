:title: nala-search

NAME
====

nala-search - Search package names and descriptions

SYNOPSIS
========

**nala search** [*--options*] REGEX ...

DESCRIPTION
===========

Search package names and descriptions using a word, regex or glob

By default **search** will only use regex.

If you start your query with *g/*, **search** will only Glob.

*Note: You may need to quote the pattern so the shell doesn't glob first*

Examples:

	Glob that will match **nala** and anything text after it:

		**sudo nala search** *"g/nala\*"*

	A question mark in a glob will match any single character:

		**sudo nala search** *"g/na?a"*

		This would match **nala** or even **nada**

Regex works similar to globbing but is much more powerful.

For more information on Glob and Regex pattern matching:

	**man** *glob.7*

	**man** *regex.7*

OPTIONS
=======

**--full**
	Print the full description of each package.

**-n**, **--names**
	Search only package names.

	This is faster than searching names and descriptions.

**-i**, **--installed**
	List only packages that are installed.

**-N**, **--nala-installed**
	List only packages explicitly installed with **nala**.

**-u**, **--upgradable**
	List only packages that are upgradable.

**-a**, **--all-versions**
	Show all versions of a package.

**-A**, **--all-arches**
	Show all architectures of a package.

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
