==========
nala-fetch
==========

--------------------------------------------
Fetch fast mirrors to improve download speed
--------------------------------------------

:Date: 23 May 2022
:Copyright: Copyright (C) 2021, 2022 Blake Lee
:Version: 0.9.1
:Manual section: 8
:Manual group: NALA

SYNOPSIS
========

``nala fetch`` [`--options`]

DESCRIPTION
===========

``nala`` will fetch fast mirrors to improve download speed.

``nala`` parses the following files to find mirrors:

	For Debian https://mirror-master.debian.org/status/Mirrors.masterlist

	For Ubuntu https://launchpad.net/ubuntu/+archivemirrors-rss

The resulting file of mirrors will be located at `/etc/apt/sources.list.d/nala-sources.list`

OPTIONS
=======

--debian
	Instead of auto detecting the distro and release, you can use this switch to specify a Debian release.

	``nala fetch`` `--debian sid`

--ubuntu
	Much like `--debian` but for Ubuntu

	``nala fetch`` `--ubuntu jammy`

--auto
	Run fetch uninteractively. Will still prompt for overwriting the sources file if it exists.

--fetches
	This controls the number of mirrors to fetch.

	With `--auto` this will default to 3 mirrors, and write as many mirrors as you specify to the file.

	In the interactive version of ``fetch``, this will default to 16 mirrors.
	Instead of controlling the amount of mirrors that will be written,
	it controls the amount of mirrors you are shown to select from.

--sources
	Add the source repos for the mirror, if it exists.

	By default ``nala`` will not include the source repos.

--foss
	Omits contrib and non-free components of the Debian mirrors.

	This switch does nothing on Ubuntu systems.

--debug
	Print helpful information for solving issues.
	If you're submitting a bug report try running the command again with `--debug`
	and providing the output to the devs, it may be helpful.

-v, --verbose
	Disable scrolling text and print extra information

-h, --help
	Shows this man page.
