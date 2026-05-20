:title: nala-clean

NAME
====

nala-clean - Clear out the local archive of downloaded package files

SYNOPSIS
========

**nala clean** [*--options*]

DESCRIPTION
===========

**nala clean** will remove the archives of package files that have been previously downloaded.

These files are typically found in */var/cache/apt/archives*.
*pkgcache.bin* and *srcpkgcache.bin* will also be removed.

OPTIONS
=======

**--lists**
	Additionally clean out the package lists normally found in */var/lib/apt/lists*

	These are the lists that are updated with **nala update**

**--fetch**
	This will remove the *nala-sources.list* file created from **nala fetch**

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
