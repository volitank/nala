# nala
======
a wrapper for the apt package manager.

# installation
==============

I have setup a repo that you can use at the moment. 

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
- nala clean
- Probably many others to add as well

**Internal**

- implement optional bandwidth check on fetch
- remove exceptions for just error messages
- setup readthedocs pages
