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
# nala is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# nala is distributed in the hope that it will be useful,
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
from subprocess import run
from typing import Dict, List, NoReturn, Optional, Union, cast

import tomli
import typer
from apt_pkg import config as apt_config

from nala import ROOT, _, __version__, color
from nala.constants import ERROR_PREFIX, GPL3_LICENSE, NOTICE_PREFIX

HookType = Dict[str, Union[str, Dict[str, Union[str, List[str]]]]]


class Config:
	"""Class for managing configurations."""

	def __init__(self) -> None:
		"""Class for managing configurations."""
		self.conf = f"{ROOT}/etc/nala/nala.conf"
		self.data: dict[str, dict[str, str | bool]] = {"Nala": {}}
		self.apt = apt_config

	def read_config(self) -> None:
		"""Read the configuration file."""
		try:
			with open(self.conf, "rb") as file:
				self.data = tomli.load(file)
		except (tomli.TOMLDecodeError, FileNotFoundError) as error:
			print(f"{ERROR_PREFIX} {error}")
			print(
				_(
					"{notice} Unable to read config file: {filename}. Using defaults"
				).format(
					notice=NOTICE_PREFIX,
					filename=color(self.conf, "YELLOW"),
					file=sys.stderr,
				)
			)

	@staticmethod
	def key_error(key: object, value: object) -> NoReturn:
		"""Exit with key error."""
		sys.exit(
			_("{error} Config key '{key}' should be a bool not {value}").format(
				error=ERROR_PREFIX, key=key, value=value
			)
		)

	def get_bool(self, key: str, default: bool = False) -> bool:
		"""Get Boolean from config."""
		value = self.data["Nala"].get(key, default)
		if isinstance(value, bool):
			return value
		self.key_error(key, value)

	def get_str(self, key: str, default: str = "") -> str:
		"""Get String from config."""
		value = self.data["Nala"].get(key, default)
		if isinstance(value, str):
			return value
		self.key_error(key, value)

	def get_hook(self, key: str) -> HookType:
		"""Get Install Hooks from config."""
		return cast(
			HookType,
			self.data.get(key, {}),
		)

	def set(self, key: str, value: str | bool) -> None:
		"""Set value in the Nala Config."""
		self.data["Nala"][key] = value

	# # This will likely need a lot of testing to be able to do something like this
	# # For now it shall remain disabled indefinitely
	# #
	# def recurse_config(self, key: str, conf: dict):
	# 	"""Recurse nala config and pass through apt options.

	# 	Example of toml apt option::

	# 	        APT.Get.AllowUnauthenticated = true
	# 	"""
	# 	for new_key, value in conf.items():
	# 		final_key = new_key
	# 		if key:
	# 			final_key = f"{key}::{new_key}"
	# 		if isinstance(value, dict):
	# 			self.recurse_config(final_key, value)
	# 			continue
	# 		if value == False:
	# 			value = "false"
	# 		elif value == True:
	# 			value = "true"
	# 		self.apt.set(final_key, value)


class Arguments:
	"""Arguments class."""

	# pylint: disable=too-many-instance-attributes, too-many-public-methods
	def __init__(self) -> None:
		"""Arguments class."""
		self.command: str = ""
		self.config = Config()
		self.config.read_config()
		# True Global
		self.verbose: bool
		self.debug: bool

		# Semi Global
		self.download_only: bool
		self.install_recommends: bool
		self.install_suggests: bool
		self.remove_essential: bool
		self.assume_yes: bool
		self.assume_no: bool
		self.update: bool
		self.raw_dpkg: bool
		self.purge: bool = False
		self.fix_broken: bool
		self.simple_summary: bool

		# Used in Show, List and Search
		self.all_versions: bool

		# Used in Search
		self.all_arches: bool

		# Used in Upgrade
		self.full_upgrade: bool

		# Search and List Arguments
		self.names: bool
		self.upgradable: bool
		self.installed: bool
		self.virtual: bool
		self.full: bool

		self.history: str | None
		self.history_id: str

		self.scroll: bool
		self.auto_remove: bool
		self.init_config()

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

	def set_summary(self, value: bool) -> None:
		"""Set option."""
		if value is None:
			self.simple_summary = self.config.get_bool("simple_summary")
			return
		self.simple_summary = value

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

	def set_full_upgrade(self, value: bool) -> None:
		"""Set option."""
		if value is None:
			self.full_upgrade = self.config.get_bool("full_upgrade")
			return
		self.full_upgrade = value

	def set_assume_prompt(self, value: Optional[bool]) -> None:
		"""Set option."""
		if value is None:
			self.assume_yes = self.config.get_bool("assume_yes")
			self.assume_no = False
		else:
			self.assume_yes = value
			self.assume_no = not value
			# If the configuration is set to true
			# -y, --assume_yes becomes a toggle to the default behavior
			if self.config.get_bool("assume_yes"):
				self.assume_yes = False

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

	def set_recommends(self, value: Optional[bool]) -> None:
		"""Set option."""
		if value is None:
			self.install_recommends = self.config.apt.find_b(
				"APT::Install-Recommends", True
			)
			return
		self.install_recommends = value
		if value:
			self.config.apt.set("APT::Install-Recommends", "1")
		if not value:
			self.config.apt.set("APT::Install-Recommends", "0")

	def set_suggests(self, value: Optional[bool]) -> None:
		"""Set option."""
		if value is None:
			self.install_suggests = self.config.apt.find_b(
				"APT::Install-Suggests", False
			)
			return
		self.install_suggests = value
		if value:
			self.config.apt.set("APT::Install-Suggests", "1")
		else:
			self.config.apt.set("APT::Install-Suggests", "0")

	def set_default_release(self, value: str) -> None:
		"""Set option."""
		if not value:
			return
		self.config.apt.set("APT::Default-Release", value)

	def set_update(self, value: Optional[bool]) -> None:
		"""Set option."""
		if value is None:
			return
		self.update = value

	def set_debug(self, value: bool) -> None:
		"""Set option."""
		self.debug = value

	def set_nala_option(self, key: str, value: str) -> None:
		"""Set Nala option."""
		if value == "false":
			option: str | bool = False
		elif value == "true":
			option = True
		else:
			option = value
		self.config.set(key.split("::", 1)[1], option)

	def set_dpkg_option(self, value: List[str]) -> List[str]:
		"""Set option."""
		if not value:
			return value
		try:
			for opt in value:
				dpkg, option = opt.split("=", 1)
				if dpkg.startswith("Nala::"):
					self.set_nala_option(dpkg, option.strip('"'))
					continue
				self.config.apt.set(dpkg, option.strip('"'))
			# Reinitialize Nala configs in case a Nala option was given
			self.init_config()
		except ValueError:
			sys.exit(
				_("{error} Option {option}: Configuration item must have a '='").format(
					error=ERROR_PREFIX, option=opt
				)
			)
		return value

	def init_config(self) -> None:
		"""Initialize Nala Configs."""
		self.scroll = self.config.get_bool("scrolling_text", True)
		self.auto_remove = self.config.get_bool("auto_remove", True)
		try:
			self.update = (
				self.config.get_bool("auto_update", True)
				if sys.argv[1] == "upgrade"
				else False
			)
		except IndexError:
			self.update = self.config.get_bool("auto_update", True)

	def state(self) -> str:
		"""Return the state of the object as a string."""
		return f"{self}"

	def is_purge(self) -> bool:
		"""Return if we are to be purging or not."""
		return bool(self.purge or "purge" in self.command)


arguments = Arguments()
nala = typer.Typer(add_completion=True, no_args_is_help=True)
history_typer = typer.Typer(name="history")
nala.add_typer(history_typer)


def print_license(value: bool) -> None:
	"""Print the GPLv3 with `--license`."""
	if not value:
		return
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


def help_callback(value: bool) -> None:
	"""Show man page instead of normal help."""
	if value:
		sys.exit(
			run(  # pylint: disable=subprocess-run-check
				["man", f"nala-{arguments.command.replace('purge', 'remove')}"]
			).returncode
		)


VERSION = typer.Option(
	False,
	"--version",
	callback=version,
	is_eager=True,
	help=_("Show program's version number and exit."),
)

LICENSE = typer.Option(
	False,
	"--license",
	callback=print_license,
	is_eager=True,
	help=_("Reads the GPLv3 which Nala is licensed under."),
)

VERBOSE = typer.Option(
	False,
	"-v",
	"--verbose",
	callback=arguments.set_verbose,
	is_eager=True,
	help=_("Disable scrolling text and print extra information."),
)

DEBUG = typer.Option(
	False,
	"--debug",
	callback=arguments.set_debug,
	is_eager=True,
	help=_("Logs extra information for debugging."),
)

AUTO_REMOVE = typer.Option(
	None,
	"--autoremove / --no-autoremove",
	callback=arguments.set_auto_remove,
	is_eager=True,
	help=_("Toggle autoremoving packages."),
)

RECOMMENDS = typer.Option(
	None,
	callback=arguments.set_recommends,
	is_eager=True,
	help=_("Toggle installing recommended packages."),
)

SUGGESTS = typer.Option(
	None,
	callback=arguments.set_suggests,
	is_eager=True,
	help=_("Toggle installing suggested packages."),
)

DEFAULT_RELEASE = typer.Option(
	None,
	"-t",
	"--target-release",
	callback=arguments.set_default_release,
	help=_("Set the default release to install packages from."),
)

FULL_UPGRADE = typer.Option(
	None,
	"--full / --no-full",
	callback=arguments.set_full_upgrade,
	help=_("Toggle full-upgrade"),
)

UPDATE = typer.Option(
	None,
	callback=arguments.set_update,
	help=_("Toggle updating the package list."),
)

SIMPLE = typer.Option(
	None,
	callback=arguments.set_summary,
	is_eager=True,
	help=_("Toggle a more condensed transaction summary."),
)

PURGE = typer.Option(
	False,
	"--purge",
	callback=arguments.set_purge,
	is_eager=True,
	help=_("Purge any packages that would be removed."),
)

CONFIG = typer.Option(
	False,
	"--config",
	help=_("Purge packages not installed that have config files."),
)

REMOVE_ESSENTIAL = typer.Option(
	False,
	"--remove-essential",
	callback=arguments.set_remove_essential,
	is_eager=True,
	help=_("Allow the removal of essential packages."),
)

FIX_BROKEN = typer.Option(
	True,
	"-f",
	"--fix-broken / --no-fix-broken",
	callback=arguments.set_fix_broken,
	is_eager=True,
	help=_("Toggle fix broken packages."),
)

ASSUME_YES = typer.Option(
	None,
	"-y / -n",
	"--assume-yes / --assume-no",
	callback=arguments.set_assume_prompt,
	is_eager=True,
	help=_("Assume 'yes' or 'no' to all prompts."),
)

OPTION = typer.Option(
	[],
	"-o",
	"--option",
	callback=arguments.set_dpkg_option,
	is_eager=True,
	help=_('Set options like Dpkg::Options::="--force-confnew".'),
)

RAW_DPKG = typer.Option(
	False,
	"--raw-dpkg",
	callback=arguments.set_raw_dpkg,
	is_eager=True,
	help=_("Skips all formatting and you get raw dpkg output."),
)

ALL_VERSIONS = typer.Option(
	False,
	"-a",
	"--all-versions",
	callback=arguments.set_all_versions,
	is_eager=True,
	help=_("Show all versions of a package."),
)

ALL_ARCHES = typer.Option(
	False,
	"-A",
	"--all-arches",
	callback=arguments.set_all_arches,
	is_eager=True,
	help=_("Show all architectures of a package."),
)

DOWNLOAD_ONLY = typer.Option(
	False,
	"-d",
	"--download-only",
	callback=arguments.set_download_only,
	is_eager=True,
	help=_("Packages are only retrieved, not unpacked or installed."),
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

NALA_INSTALLED = typer.Option(
	False,
	"-N",
	"--nala-installed",
	is_eager=True,
	help=_("Only packages explicitly installed with Nala."),
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

MAN_HELP = typer.Option(
	False,
	"-h",
	"--help",
	callback=help_callback,
	is_eager=True,
	help=_("Show this message and exit."),
)

CONTEXT_SETTINGS = {
	"help_option_names": ["-h", "--help"],
}


@nala.command("help", hidden=True)
@nala.callback(
	context_settings=CONTEXT_SETTINGS, no_args_is_help=True, invoke_without_command=True
)
# pylint: disable=unused-argument
def global_options(
	ctx: typer.Context,
	_version: bool = VERSION,
	_license: bool = LICENSE,
) -> None:
	"""Each command has its own help page.

	For Example: `nala history --help`
	"""
	if ctx.invoked_subcommand:
		arguments.command = ctx.invoked_subcommand
		if arguments.command == "help":
			print(ctx.get_help())
			sys.exit()
