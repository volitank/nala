:title: Nala

NAME
====

Nala - Front-end for libapt-pkg

SYNOPSIS
========

**nala** <*command*> [*--options*] PKGS ...

DESCRIPTION
===========

**nala** is a prettier front-end for libapt-pkg.

The goals of **nala** are to be similar to **apt** but add quality of life changes,
and improve the output to make it more reader friendly.

COMMANDS
========

**install**

	**nala** takes multiple packages as arguments and will install all of them just like **apt**.

	**nala** downloads packages in parallel.
	Along with this **nala** can download packages from multiple mirrors concurrently to speed up downloads further.
	We put in place a limit to 3 packages per mirror so we don't put too much pressure on them.
	This caps out at a maximum of 16.

	For more information and options run **nala install** *--help*

**remove**

	**remove** works similar to the way it does in **apt**, but **nala** will also autoremove packages.

	For more information and options run **nala remove** *--help*

**purge**

	**purge** works similar to **remove**, but also gets rid of configuration files.

	For more information and options run **nala purge** *--help*

**update**

	**update** updates the package list. Works exactly like **apt update** with better formatting.

**upgrade**

	**upgrade** upgrades packages installed on the system.

	By default **nala** will run the equivalent of **apt update && apt full-upgrade** *--auto-remove*.

	This behavior can be changed in the configuration file or you can use **nala upgrade** *--no-update*.

	For more information and options run **nala upgrade** *--help*

**autoremove**

	**autoremove** removes any packages that are no longer needed.

	For more information and options run **nala autoremove** *--help*

**autopurge**

	**autopurge** purges any packages that are no longer needed.

	For more information and options run **nala autopurge** *--help*

**clean**

	**clean** will remove any *.deb* files in the archive directory, typically */var/cache/apt/archives*

	For more information and options run **nala clean** *--help*

**fetch**

	**fetch** is the first command that doesn't have an **apt** counterpart.

	**nala** will parse either:
		the **Debian** mirror list from *https://mirror-master.debian.org/status/Mirrors.masterlist*,

		or the **Ubuntu** mirror list from *https://launchpad.net/ubuntu/+archivemirrors-rss*

	**nala** will then select the mirrors that it has determined are the fastest.

	This is done by downloading the Release file and timing how long it takes.

	For more information and options run **nala fetch** *--help*

**show**

	**show** information about the given package(s) highlighted with color to improve readability.

	For more information and options run **nala show** *--help*

**search**

	**search** all packages in the cache with a regex string.
	By default searches package names, source names, and descriptions.

	For more information and options run **nala search** *--help*

**list**

	**list** packages in the cache.

	For more information and options run **nala list** *--help*

**history**

	**history** is another command not found in **apt**.
	Every **install**, **remove**, or **upgrade** command is stored with an id.
	You can use **history** to view these in a summary style view, and even go more in depth with **history info [id]**.
	If you're familiar with how *Fedora*'s **dnf history** command works, then you'll feel right at home.

	This command is inspired by **dnf**'s history functionality.

	For more information and options run **nala history** *--help*

EXAMPLES
========

**nala install** *--update* **wine**
	downloads and installs wine, updating the package cache first.

**nala upgrade** *--no-update*
	upgrades the system without updating the package cache.

**nala fetch** *--auto --fetches* **8** *--country* **US**
	fetches 8 mirrors in the United States uninteractively.

BUGS
====

Submit bug reports and feature requests online at:

	<*https://gitlab.com/volian/nala/-/issues*>

SEE ALSO
========

Sources at: <*https://gitlab.com/volian/nala*>

AUTHORS
=======

Blake Lee <*https://gitlab.com/volitank*> <*blake@volian.org*>

volian-team <*https://gitlab.com/volian*> <*volian-devel@volian.org*>

COPYRIGHT
=========

Copyright (C) 2021, 2022 Blake Lee
