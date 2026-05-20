:title: nala-update

NAME
====

nala-update - Update the list of available packages

SYNOPSIS
========

**nala update** [*--options*]

DESCRIPTION
===========

**nala** will update the list of available packages.

This command is identical to the **apt** variant, but the output is formatted differently

OPTIONS
=======

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

COPYRIGHT
=========

Copyright (C) 2021, 2022 Blake Lee
