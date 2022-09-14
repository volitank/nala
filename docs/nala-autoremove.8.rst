:title: nala-autoremove/autopurge

NAME
====

nala-autoremove/autopurge - Autoremove or Autopurge packages that are no longer needed

SYNOPSIS
========

**nala autoremove** [*--options*]

**nala autopurge** [*--options*]

DESCRIPTION
===========

Automatically remove or purge any packages that are no longer needed.

Packages that are no longer needed are usually dependencies that were
automatically installed and no package depends on them any longer.

The **autoremove** and **autopurge** documentation are combined as they are almost the same command.

**nala autopurge** is really just **nala autoremove** *--purge*

OPTIONS
=======

**--config**
	Purge configuration files for packages that are no longer installed.

**--purge**
	Purge any packages that would removed during the transaction.

**--debug**
	Print helpful information for solving issues.
	If you're submitting a bug report try running the command again with *--debug*
	and providing the output to the devs, it may be helpful.

**--raw-dpkg**
	Force **nala** not to format **dpkg** output.

	This disables all formatting and it would look as if you were using **apt**.

	A more indepth explanation for what this switch does,
	**nala** will fork a tty instead of a pty for **dpkg**.

	**nala** will also not display a progress bar for **dpkg** with this turned on.
	Additionally the language of the output will not be forced into English for this mode.

**-d**, **--download-only**
	Packages are only retrieved, not unpacked or installed.

**--remove-essential**
	Allow the removal of essential packages.

	This is very dangerous, but we thought you should have the option.

**--assume-yes**, **--assume-no**

	*-y*, *--assume-yes*

		Automatically select *yes* for any prompts which may need your input.

		If the configuration option *assume_yes* is true, this switch will
		set it back to default behavior

	*-n*, *--assume-no*

		Automatically select *no* for any prompts which may need your input.

**--simple**, **--no-simple**

	*--simple*

		Show a more simple and condensed transaction summary.

	*--no-simple*

		Show the standard table transatction summary with more information.

		This variant is the **default**

**-o**, **--option**
	Set options to pass through to **apt**, **nala**, or **dpkg**.

	Example:
		Force **dpkg** to install new config files without prompting:

			**nala install --option** *Dpkg::Options::="--force-confnew"*

		Disable scrolling text for **nala**

			**nala install --option** *Nala::scrolling_text="false"*

		Allow **nala** to **update** unauthenticated repositories:

			**nala install --option*** *APT::Get::AllowUnauthenticated="true"*

**-v**, **--verbose**
	Disable scrolling text and print extra information

**-h**, **--help**
	Shows this man page.

**--update**, **--no-update**
	*--update*

		Update the package list before the requested operation.

		Example:

			**nala install** *--update* **neofetch**

			is equivalent to

			**apt update** && **apt install neofetch**

		[Default for: **upgrade**]

	*--no-update*

		Do **NOT** update the package list before the requested operation.

		[Default for: **install**, **remove**, **purge**, **autoremove**, **autopurge**]

**--fix-broken**, **--no-fix-broken**
	*--fix-broken*

		Attempts to fix broken packages.

		This variant is the **default**

	*--no-fix-broken*

		Stops **nala** from performing extra checks.

		*This can result in a broken install*!

	If you just want to fix broken packages:

		**nala install** *--fix-broken*

COPYRIGHT
=========

Copyright (C) 2021, 2022 Blake Lee
