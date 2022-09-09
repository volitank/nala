:title: nala-fetch

NAME
====

nala-fetch - Fetch fast mirrors to improve download speed

SYNOPSIS
========

**nala fetch** [*--options*]

DESCRIPTION
===========

**nala** will fetch fast mirrors to improve download speed.

**nala** parses the following files to find mirrors:

	For Debian https://mirror-master.debian.org/status/Mirrors.masterlist

	For Ubuntu https://launchpad.net/ubuntu/+archivemirrors-rss

	For Devuan https://pkgmaster.devuan.org/mirror_list.txt

The resulting file of mirrors will be located at */etc/apt/sources.list.d/nala-sources.list*

When selecting mirrors in the interactive display, you can pass a comma or space separated list.

Examples:

	Specific List:

		*1,2,3,4* Selects mirrors 1, 2, 3 and 4

		Comma and spaces may be mixed in the input. **nala** will parse it all the same.

		*1,3 , 4 5* will select mirrors 1, 3, 4 and 5

	Range: *1..5* will select mirrors 1 through 5

	Even Range: *0..1..5* selects 2 and 4

	Odd Range: *1..1..5* selects 1, 3 and 5

OPTIONS
=======

**--debian**
	Instead of auto detecting the distro and release, you can use this switch to specify a Debian release.

	**nala fetch** *--debian sid*

**--ubuntu**
	Much like *--debian* but for Ubuntu

	**nala fetch** *--ubuntu jammy*

**--devuan**
	Much like *--debian* but for Devuan

	**nala fetch** *--devuan unstable*

**--https-only**
	**fetch** will only provide https mirrors.

**--auto**
	Run **fetch** uninteractively. Will still prompt for overwriting the sources file if it exists.

**--fetches**
	This controls the number of mirrors to fetch.

	With *--auto* this will default to 3 mirrors, and write as many mirrors as you specify to the file.

	In the interactive version of **fetch**, this will default to 16 mirrors.
	Instead of controlling the amount of mirrors that will be written,
	it controls the amount of mirrors you are shown to select from.

**--sources**
	Add the source repos for the mirror, if it exists.

	By default **nala** will not include the source repos.

**--non-free**
	Add contrib and non-free components of the Debian mirrors.

	This switch does nothing on Ubuntu systems.

**--debug**
	Print helpful information for solving issues.
	If you're submitting a bug report try running the command again with *--debug*
	and providing the output to the devs, it may be helpful.

**-c**, **--country**
	Specify your country to limit the mirror search using the 2 letter ISO country code.

	This option can be specified multiple times to increase the pool.

	Example:

		**nala fetch** *-c US -c CA*

**-v**, **--verbose**
	Disable scrolling text and print extra information

**-h**, **--help**
	Shows this man page.

COPYRIGHT
=========

Copyright (C) 2021, 2022 Blake Lee
