.. image:: https://img.shields.io/discord/923757419253882920?color=5865F2&label=Discord&logo=discord&logoColor=FFFFFF&style=flat-square
	:target: https://discord.gg/JEFpg73yr7
	:alt: Discord
.. image:: https://app.codacy.com/project/badge/Grade/686108742fe042c6b31965b5cf51a042
	:target: https://www.codacy.com/gl/volian/nala/dashboard?utm_source=gitlab.com&amp;utm_medium=referral&amp;utm_content=volian/nala&amp;utm_campaign=Badge_Grade
	:alt: Codacy

.. contents:: Table of Contents
	:depth: 1
	:local:
	:backlinks: none

# Nala
======

Nala is a front-end for ``libapt-pkg``. Specifically we interface using the ``python-apt`` api.

Especially for newer users it can be hard to understand what ``apt`` is trying to do when installing or upgrading.

We aim to solve this by not showing some redundant messages, formatting the packages better, and using color to
show specifically what will happen with a package during install, removal, or an upgrade.

# Installation
==============

For installation instructions see our `wiki page <https://gitlab.com/volian/nala/-/wikis/Installation>`_.

# Parallel Downloads
====================

Outside of pretty formatting, the number 1 reason to use Nala over ``apt`` is parallel downloads.

By default we will download 3 packages per unique mirror in your ``sources.list`` file.

Opening multiple connections to the same mirror is great for speeding up downloading many small packages.
We have the 3 connections per mirror limit to minimize how hard we are hitting mirrors.

Additionally we alternate downloads between the available mirrors to improve download speeds even further.
If a mirror fails for whatever reason, we just try the next until all defined mirrors are exhausted.

`Note: Nala does not use APT for package downloading and verification`

# Fetch
=======

Which brings us to our next standout feature, ``nala fetch``.

This command works similar to how most people use ``netselect`` and ``netselect-apt``.
``nala fetch`` will check if your distro is either Debian or Ubuntu.
Nala will then go get all the mirrors from the respective master list.
Once done we test the latency and score each mirror.
Nala will choose the fastest 3 mirrors (configurable) and write them to a file.

`At the moment fetch will only work on Debian, Ubuntu and derivatives still tied to the main repos. Such as Pop!_OS`

# History
=========

Our last big feature is the ``nala history`` command.

If you're familiar with ``dnf`` this works much in the same way.
Each Install, Remove or Upgrade we store in /var/lib/nala/history.json with a unique ``<ID>`` number.

At any time you can call ``nala history`` to print a summary of every transaction ever made.
You can then further manipulate this with commands such as ``nala history undo <ID>`` or ``nala history redo <ID>``.

If there is something in the history file that you don't want you can use the ``nala history clear <ID>`` It will remove that entry.
Alternatively for the ``clear`` command we accept ``--all`` which will remove the entire history.

# Zsh/fish Completions
======================

Nala's bash, Zsh and fish completions are now handled with ``typer``.

There is nothing you need to do but install Nala and restart your shell for them to work

# Bug Reports or Feature Requests
=================================

Nala is mirrored to several sites such as GitHub and even Debian Salsa.

The official repository is https://gitlab.com/volian/nala

We ask that you please go here to report a bug or request a feature.

The other repositories are official, but just mirrors of what is on GitLab.

# Donations
===========

If you would like to support the project you can donate at the link below.

https://liberapay.com/Volian-Linux

# Images
========

.. image:: /imgs/nala-install-1.png
.. image:: /imgs/nala-install-2.png

.. image:: /imgs/nala-fetch.png

.. image:: /imgs/nala-history-info.png
.. image:: /imgs/nala-history-undo-1.png
.. image:: /imgs/nala-history-undo-2.png

.. image:: /imgs/nala-update.png
.. image:: /imgs/nala-show-apt.png
