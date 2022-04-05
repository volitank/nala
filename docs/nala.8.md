---
title: Nala
section: 8
header: User Manual
footer: Nala 0.7.3
date: 4 April 2022
---
# NAME
Nala - A prettier front-end for libapt-pkg

# SYNOPSIS
**nala** <*command*> [*\--options*]...

# DESCRIPTION
**nala** is a prettier front-end for libapt-pkg. The goals of **nala** are to be similar to **apt** but add quality of life changes, and improve the output to make it more reader friendly.

**install**
: > **install** works similar to the way it does in **apt**. **nala** takes multiple packages as arguments and will install all of them just like **apt**.

: > **nala** downloads packages in parallel. Along with this **nala** can download packages from multiple mirrors concurrently to speed up package downloads further. We put in place a limit to 2 packages per mirror so we don't put too much pressure on them. This caps out at a maximum of 16.

	**-f, \--fix-broken** attempts to fix broken packages. By default Nala will attempt to fix them. This switch exists so if you install something with **dpkg** and it needs extra dependencies to work you can run **nala install --fix-broken**.

**remove**
: > **remove** works similar to the way it does in **apt**. Our noticeable differences here include improved output on what will be removed, no need for running an autoremove, **nala** will handle that for you.

**purge**
: > **purge** works similar to **remove**, but also gets rid of configuration files.

**update**
: > **update** updates the package list. Works exactly like **apt update** with better formatting.

**upgrade**
: > **upgrade** upgrades packages installed on the system.

	By default **nala** will run the equivalent of **apt update && apt full-upgrade --auto-remove**. If you are just looking to update the package cache and not actually perform an upgrade you can use **nala** *\--update*.

	*\--no-full* will run a regular upgrade which won't remove packages. By default **nala** uses a *full-upgrade*.

	*\--exclude* packages to exclude when updating. Glob enabled, for example. **nala update** *\--exclude* *linux-image**

**autoremove**, **autopurge**
: > **autoremove/purge** removes or purges packages that are no longer needed.

**clean**
: > **clean** is exactly like running *apt clean*.

	*\--fetch* Remove `nala-sources.list` generated from **nala fetch**

	*\--lists* Remove package lists located in `/var/lib/apt/lists/`

**fetch**
: > **fetch** is our first command that doesn't have an **apt** counterpart.

	**nala** will parse either the **Debian** mirror list from *https://mirror-master.debian.org/status/Mirrors.masterlist*, or the **Ubuntu** mirror list from *https://launchpad.net/ubuntu/+archivemirrors-rss* and then fetch (3 by default) mirrors that we have determined are the closest to you.

	**nala** will attempt to detect your distro and release by default. Don't worry if it's not able too, as you can specify it manually with some switches below.

	This functionality is much like you would expect from **netselect** and **netselect-apt**.

	*\--debian* lets you specify that you're using **Debian** and what release you're using. *\--debian sid*

	*\--ubuntu* This is just the **Ubuntu** version of the switch above. *\--ubuntu jammy*

	*\--country* lets you specify your *country* when fetching mirrors. Use the ISO country code. You don't have to use this as we test latency anyway, but it can speed things up as there are less mirrors to test.

	*\--foss* on *Debian* will ensure that you don't get the *contrib* or *non-free* repos. Using this on *Ubuntu* does nothing.

	*\--sources* add the source repos for the mirrors if it exists.

	*\--count* choose the number of viable mirrors to display. by default it's 16, but if you want more you got it.

	*\--auto* run fetch uninteractively. Will still prompt for overwrite.

	*\--fetches* lets you determine the amount of mirrors to fetch between 1-10. 3 is the default. This switch only works with *\--auto*

**show**
: > **show** works exactly like the **apt** version except our output is a little easier to read. **show** will accept multiple packages as arguments.

	*\-a, \--all-versions* will print information about each version that is currently in the cache.

**search**
: > **search** all packages in the cache with a regex string. By default searches package names, source names, and descriptions.

	*\-n, \--names* will search only package names. This is much faster than searching everything.

	*\-i, \--installed* will search only installed packages.

	*\-u, \--upgradable* will search only upgradable packages.

	*\-a, \--all-versions* will print information about each version that is currently in the cache.

	*\-A, \--all-arches* will search all architectures of a package.

	*\-V, \--virtual* will search only virtual packages.

	*\--full* will print the full description of each package found rather than just the short description.

**list**
: > **list** packages in the cache.

	The switches here are the same as the **search** command except *\-n, \--names* and *\-A, \--all-arches* are absent.

**history**
: > **history** is our other new command. Every **install**, **remove**, or **upgrade** command is stored with an id. You can use **history** to view these in a summary style view, and even go more in depth with **history info [id]**. If you're familiar with how *Fedora's* **dnf history** command works, then you'll feel right at home. That's what we drew inspiration from.

	**history info [id]** prints a more detailed summary of what happened in that transaction.

	**history undo [id]** will undo the transaction.

	**history redo [id]** will redo the transaction.

	Currently only **remove** and **install** commands are available for **undo** and **redo**

	**history clear [id]** will remove the transaction from the history. Alternatively you can use "all" for the ID which will remove the entire history

# OPTIONS
**\--help**
: >*\--help* will print out a help message for each subcommand. **nala install** *\--help* is a different message than **nala update** *\--help*.

**-y, \--assume-yes**
: >*\--assume-yes* will automatically select yes for any prompts which may need your input. This can potentially be dangerous.

**-d, \--download-only**
: >*\--download-only* will do just that, download packages only. It will not unpack or configure anything.

**-v, \--verbose**
: >*\--verbose* prints more information that sticks around. By default we use "Progress Bars" to hide some things we consider unnecessary. Using *\--verbose* removes the progress bars and prints that information.

**-o, \--option**
: >*\--option* can pass any configuration option to **apt**/**dpkg** like Dpkg::Options::="--force-confnew" or APT::Default-Release="stable".

**\--purge**
: >*\--purge* any packages that would removed during the transaction.

**\--fix-broken** / **\--no-fix-broken**
: >*\--fix-broken* / *\--no-fix-broken/* toggles for fixing broken packages

	*\--fix-broken* will attempt to fix broken packages. **nala install** *\--fix-broken*

	*\--no-fix-broken* can be used if you don't want Nala to attempt to fix broken packages.

**\--install-recommends** / **\--no-install-recommends**
: >*\--install-recommends* / *\--no-install-recommends* toggles for recommended packages

	*\--install-recommends** sets recommended packages to be installed. [Default]

	*\--no-install-recommends* stops the installation of recommended packages. If this option is selected Nala will display the recommended packages that won't be installed.

**\--install-suggests** / **\--no-install-suggests**
: >*\--install-suggests* / *\--no-install-suggests* toggles for suggested packages

	*\--install-suggests* installs suggested package. If this option is not selected Nala will display the suggested packages that won't be installed.

	*\--no-install-suggests* suggested packages will not be installed. [Default]

**\--update** / **\--no-update**
: >*\--update* / *\--no-update* toggles for updating the package list

	*\--no-update* skips updating the package cache if for whatever reason you would like to skip that.

	*\--update* updates the package cache. Typically run as **nala install** *\--update* or **nala** *\--update*.

**\--autoremove** / **\--no-autoremove**
: >*\--autoremove* / *\--no-autoremove* toggles for autoremoving packages

	*\--autoremove* nala will autoremove unneeded packages. [Default]

	*\--no-autoremove* stops nala from autoremoving packages. Packages that need to be removed in order to continue the upgrade will still be removed.

**\--remove-essential**
: >*\--remove-essential* allows the removal of essential packages. This is very dangerous, but we thought you should have the option.

**\--debug**
: >*\--debug* prints helpful information for solving issues. If you're submitting a bug report try running the command again with *\--debug* and providing the output to the devs, it will be helpful.

**\--version**
: >*\--version* prints the version of nala you have installed and exits.

**\--license**
: >*\--license* reads the licenses of software compiled in and then reads the GPLv3 which **nala** is licensed under.

**\--raw-dpkg**
: >*\--raw-dpkg* forces **nala** not to mess with **dpkg** while running operations. This disables all formatting and it would look as if you were using **apt**. A more indepth explanation for what this switch does, **nala** will fork a tty instead of a pty for **dpkg**.

	**nala** will also not display a progress bar for **dpkg** with this turned on. Additionally the language of the output will not be forced into English for this mode.

# EXAMPLES
**nala install** *\--update* **wine**
: >downloads and installs wine, updating the package cache first.

**nala upgrade** *\--no-update*
: >upgrades the system without updating the package cache.

**nala fetch** *\--fetches* **8** *\--country* **US**
: >fetches 8 mirrors in the United states to max out parallel downloads.

# AUTHORS
Blake Lee <*https://gitlab.com/volitank*> <*blake@volian.org*>

volian-team <*https://gitlab.com/volian*> <*volian-devel@volian.org*>

# BUGS
Submit bug reports online at: <*https://gitlab.com/volian/nala/-/issues*>

# SEE ALSO
Sources at: <*https://gitlab.com/volian/nala*>
