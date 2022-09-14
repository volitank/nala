:title: nala-remove/purge

NAME
====

nala-remove/purge - Remove or purge packages that are no longer needed

SYNOPSIS
========

**nala remove** [*--options*] PKGS ...

**nala purge** [*--options*] PKGS ...

DESCRIPTION
===========

Remove or purge packages that are no longer needed

The **remove** and **purge** documentation are combined as they are almost the same command.

**nala purge** is really just **nala remove** *--purge*

**nala** will also, by default, remove any packages which are no longer needed.

OPTIONS
=======

**--purge**
	Purge any packages that would be removed during the transaction.

	**nala remove** *--purge* is equivalent to **nala purge**

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

**--autoremove**, **--no-autoremove**
	*--autoremove*

		Automatically remove any packages that are no longer needed.

		This variant is the **default**

	*--no-autoremove*

		Do **NOT** Automatically remove any packages

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

**--install-recommends**, **--no-install-recommends**
	*--install-recommends*

		Recommended packages will be installed.

		This variant is the **default** unless changed with the **apt** config.

	*--no-install-recommends*

		Recommended package will **NOT** be installed.

		If this option is selected **nala** will display the recommended packages that will not be installed.

**--install-suggests**, **--no-install-suggests**
	*--install-suggests*

		Suggested packages will be installed.

		This variant is the **default**

	*--no-install-suggests*

		This variant is the **default** unless changed with the **apt** config.

		If this option is selected **nala** will display the suggested packages that will not be installed.

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
