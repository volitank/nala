#                 __
#    ____ _____  |  | _____
#   /    \\__  \ |  | \__  \
#  |   |  \/ __ \|  |__/ __ \_
#  |___|  (____  /____(____  /
#       \/     \/          \/
#
# Copyright (C) 2021, 2022 Blake Lee
#
# This file is part of nala
#
# nala is program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# nala is program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with nala.  If not, see <https://www.gnu.org/licenses/>.
"""The options module."""
from __future__ import annotations

import sys
from pydoc import pager

import typer
from apt_pkg import Error, config, read_config_file

from nala import _, __version__, color
from nala.constants import (
	ERROR_PREFIX,
	GPL3_LICENSE,
	NOTICE_PREFIX,
	THIRD_PARTY_LICENSES,
)

CONF_FILE = "/etc/nala/nala.conf"

try:
	read_config_file(config, CONF_FILE)
except Error as error:
	print(
		str(error)
		.replace("E:", f"{ERROR_PREFIX} ")
		.replace(CONF_FILE, color(CONF_FILE, "YELLOW")),
		file=sys.stderr,
	)
	sys.exit(1)
except SystemError:
	print(
		_("{notice} Unable to read config file: {filename}. Using defaults").format(
			notice=NOTICE_PREFIX, filename=color(CONF_FILE, "YELLOW"), file=sys.stderr
		)
	)

no_update = (
	"remove",
	"show",
	"search",
	"list",
	"history",
	"install",
	"purge",
	"autoremove",
	"autopurge",
)


class Arguments:
	"""Arguments class."""

	# pylint: disable=too-many-instance-attributes
	def __init__(self) -> None:
		"""Arguments class."""
		self.command: str = ""
		# True Global
		self.verbose: bool
		self.debug: bool

		# Semi Global
		self.download_only: bool
		self.install_recommends: bool
		self.install_suggests: bool
		self.remove_essential: bool
		self.assume_yes: bool
		self.update: bool
		self.raw_dpkg: bool
		self.purge: bool
		self.fix_broken: bool

		# Used in Show, List and Search
		self.all_versions: bool

		# Used in Search
		self.all_arches: bool

		# Search and List Arguments
		self.names: bool
		self.upgradable: bool
		self.installed: bool
		self.virtual: bool
		self.full: bool

		self.history: str
		self.history_id: str

		self.scroll = config.find_b("Nala::ScrollingText", True)
		self.auto_remove = config.find_b("Nala::AutoRemove", True)

	def __str__(self) -> str:
		"""Return the state of the object as a string."""
		kwarg = "\n    ".join(
			(f"{key} = {value},") for key, value in self.__dict__.items()
		)
		return f"Options = [\n    {kwarg}\n]"

	def set_verbose(self, value: bool) -> None:
		"""Set option."""
		if not value:
			self.verbose = False
			return
		self.verbose = True
		self.scroll = False

	def set_auto_remove(self, value: bool) -> None:
		"""Set option."""
		if value is None:
			return
		self.auto_remove = value

	def set_purge(self, value: bool) -> None:
		"""Set option."""
		self.purge = value

	def set_remove_essential(self, value: bool) -> None:
		"""Set option."""
		self.remove_essential = value

	def set_download_only(self, value: bool) -> None:
		"""Set option."""
		self.download_only = value

	def set_fix_broken(self, value: bool) -> None:
		"""Set option."""
		self.fix_broken = value

	def set_assume_yes(self, value: bool) -> None:
		"""Set option."""
		self.assume_yes = value

	def set_raw_dpkg(self, value: bool) -> None:
		"""Set option."""
		self.raw_dpkg = value

	def set_all_versions(self, value: bool) -> None:
		"""Set option."""
		self.all_versions = value

	def set_all_arches(self, value: bool) -> None:
		"""Set option."""
		self.all_arches = value

	def set_names(self, value: bool) -> None:
		"""Set option."""
		self.names = value

	def set_installed(self, value: bool) -> None:
		"""Set option."""
		self.installed = value

	def set_upgradable(self, value: bool) -> None:
		"""Set option."""
		if not value and hasattr(self, "upgradable"):
			return
		self.upgradable = value

	def set_virtual(self, value: bool) -> None:
		"""Set option."""
		self.virtual = value

	def set_full(self, value: bool) -> None:
		"""Set option."""
		self.full = value

	def set_recommends(self, value: bool) -> None:
		"""Set option."""
		if value is None:
			self.install_recommends = config.find_b("APT::Install-Recommends", True)
			return
		self.install_recommends = value
		if value:
			config.set("APT::Install-Recommends", "1")
		if not value:
			config.set("APT::Install-Recommends", "0")

	def set_suggests(self, value: bool) -> None:
		"""Set option."""
		if value is None:
			self.install_suggests = config.find_b("APT::Install-Suggests", False)
			return
		self.install_suggests = value
		if value:
			config.set("APT::Install-Suggests", "1")
		if not value:
			config.set("APT::Install-Suggests", "0")

	def set_update(self, value: bool) -> None:
		"""Set option."""
		if value is None:
			self.update = self.command not in no_update
			return
		if value:
			self.update = True
		if not value:
			self.update = False

	def set_debug(self, value: bool) -> None:
		"""Set option."""
		self.debug = value

	def state(self) -> str:
		"""Return the state of the object as a string."""
		return str(self)


arguments = Arguments()
nala = typer.Typer(add_completion=False, no_args_is_help=True)
history_typer = typer.Typer(name="history", add_completion=False)
nala.add_typer(history_typer)


def _doc(obj: object) -> object:
	"""Translate the docstring for typer help."""
	if not obj.__doc__:
		return obj
	obj.__doc__ = _(obj.__doc__)
	return obj


def print_license(value: bool) -> None:
	"""Print the GPLv3 with `--license`."""
	if not value:
		return
	pager(THIRD_PARTY_LICENSES)
	if GPL3_LICENSE.exists():
		with open(GPL3_LICENSE, encoding="utf-8") as file:
			pager(file.read())
	else:
		print(
			_(
				"It seems the system has no license file\n"
				"The full GPLv3 can be found at:\n"
				"https://www.gnu.org/licenses/gpl-3.0.txt"
			)
		)
	sys.exit()


def version(value: bool) -> None:
	"""Print version."""
	if not value:
		return
	print(f"nala {__version__}")
	sys.exit()


@nala.command("help", hidden=True)
def _help() -> None:
	...


VERSION = typer.Option(
	False,
	"--version",
	callback=version,
	is_eager=True,
	help=_("Show program's version number and exit"),
)

LICENSE = typer.Option(
	False,
	"--license",
	callback=print_license,
	is_eager=True,
	help=_("Reads the licenses of software compiled in and then reads the GPLv3"),
)

VERBOSE = typer.Option(
	False,
	"-v",
	"--verbose",
	callback=arguments.set_verbose,
	is_eager=True,
	help=_("Disable scrolling text and print extra information"),
)

DEBUG = typer.Option(
	False,
	"--debug",
	callback=arguments.set_debug,
	is_eager=True,
	help=_("Logs extra information for debugging"),
)

AUTO_REMOVE = typer.Option(
	None,
	callback=arguments.set_auto_remove,
	is_eager=True,
	help=_("Toggle autoremoving packages"),
)

RECOMMENDS = typer.Option(
	None,
	callback=arguments.set_recommends,
	is_eager=True,
	help=_("Toggle installing recommended packages"),
)

SUGGESTS = typer.Option(
	None,
	callback=arguments.set_suggests,
	is_eager=True,
	help=_("Toggle installing suggested packages"),
)

UPDATE = typer.Option(
	None,
	callback=arguments.set_update,
	is_eager=True,
	help=_("Toggle updating the package list"),
)

PURGE = typer.Option(
	False,
	"--purge",
	callback=arguments.set_purge,
	is_eager=True,
	help=_("Purge any packages that would be removed"),
)

REMOVE_ESSENTIAL = typer.Option(
	False,
	"--remove-essential",
	callback=arguments.set_remove_essential,
	is_eager=True,
	help=_("Allow the removal of essential packages"),
)

DOWNLOAD_ONLY = typer.Option(
	False,
	"--download-only",
	callback=arguments.set_download_only,
	is_eager=True,
	help=_("Packages are only retrieved, not unpacked or installed"),
)

FIX_BROKEN = typer.Option(
	True,
	callback=arguments.set_fix_broken,
	is_eager=True,
	help=_("Attempt to fix broken packages"),
)

ASSUME_YES = typer.Option(
	False,
	"-y",
	"--assume-yes",
	callback=arguments.set_assume_yes,
	is_eager=True,
	help=_("Assume 'yes' to all prompts"),
)

RAW_DPKG = typer.Option(
	False,
	"--raw-dpkg",
	callback=arguments.set_raw_dpkg,
	is_eager=True,
	help=_("Skips all formatting and you get raw dpkg output"),
)

ALL_VERSIONS = typer.Option(
	False,
	"-a",
	"--all-versions",
	callback=arguments.set_all_versions,
	is_eager=True,
	help=_("Show all versions of a package"),
)

ALL_ARCHES = typer.Option(
	False,
	"-A",
	"--all-arches",
	callback=arguments.set_all_arches,
	is_eager=True,
	help=_("Show all architectures of a package"),
)

DOWNLOAD_ONLY = typer.Option(
	False,
	"--download-only",
	callback=arguments.set_download_only,
	is_eager=True,
	help=_("Packages are only retrieved, not unpacked or installed"),
)

NAMES = typer.Option(
	False,
	"-n",
	"--names",
	callback=arguments.set_names,
	is_eager=True,
	help=_("Search only package names."),
)

INSTALLED = typer.Option(
	False,
	"-i",
	"--installed",
	callback=arguments.set_installed,
	is_eager=True,
	help=_("Only installed packages."),
)

UPGRADABLE = typer.Option(
	False,
	"-u",
	"--upgradable",
	callback=arguments.set_upgradable,
	is_eager=True,
	help=_("Only upgradable packages."),
)

UPGRADEABLE = typer.Option(
	False,
	"--upgradeable",
	callback=arguments.set_upgradable,
	is_eager=True,
	hidden=True,
)

VIRTUAL = typer.Option(
	False,
	"-V",
	"--virtual",
	callback=arguments.set_virtual,
	is_eager=True,
	help=_("Only virtual packages."),
)

FULL = typer.Option(
	False,
	"--full",
	callback=arguments.set_full,
	is_eager=True,
	help=_("Print the full description of each package."),
)

LISTS = typer.Option(
	False,
	"--lists",
	help=_("Remove package lists located in `/var/lib/apt/lists/`."),
)

FETCH = typer.Option(
	False,
	"--fetch",
	help=_("Remove `nala-sources.list`."),
)

AUTO = typer.Option(
	False, help=_("Run fetch uninteractively. Will still prompt for overwrite")
)

CONTEXT_SETTINGS = {
	"help_option_names": ["-h", "--help"],
}


@nala.callback(
	context_settings=CONTEXT_SETTINGS, no_args_is_help=True, invoke_without_command=True
)
# pylint: disable=unused-argument
def global_options(
	ctx: typer.Context,
	_version: bool = VERSION,
	_license: bool = LICENSE,
	debug: bool = DEBUG,
	verbose: bool = VERBOSE,
) -> None:
	"""Each command has its own help page.

	For Example: `nala history --help`
	"""
	if ctx.invoked_subcommand:
		arguments.command = ctx.invoked_subcommand
		if arguments.command == "help":
			print(ctx.get_help())
			sys.exit()
