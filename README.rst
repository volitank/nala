# nala
======
a wrapper for the apt package manager.

# installation
==============

Install the Volian Scar repo and then install Nala

.. code-block:: console
 
	$ echo "deb http://deb.volian.org/volian/ scar main" | sudo tee /etc/apt/sources.list.d/volian-archive-scar-unstable.list
	$ wget -qO - https://deb.volian.org/volian/scar.key | sudo tee /etc/apt/trusted.gpg.d/volian-archive-scar-unstable.gpg > /dev/null
	$ sudo apt update && sudo apt install nala

There isn't a documentation site setup at the moment, but our man page explains things well enough for now.

# todo
======

**Commands and Switches**

- -f --fix-broken
- --no-install-recommends
- --install-suggests
- --no-upgrade
- nala download
- nala purge
- switch to silently upgrade (Auto Select Config file changes, etc)
- Probably many others to add as well

**Internal**

- implement optional bandwidth check on fetch
- remove exceptions for just error messages
- setup readthedocs pages
