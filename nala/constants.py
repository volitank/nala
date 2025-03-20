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
"""Module for file constants."""
from __future__ import annotations

import re
from enum import IntEnum
from pathlib import Path
from signal import Handlers  # pylint: disable=no-name-in-module #Codacy
from types import FrameType
from typing import Any, Callable, Optional, Union

import apt_pkg

from nala import ROOT, USR, _, color

# File Constants
GPL3_LICENSE = Path(f"{USR}/share/common-licenses/GPL-3")
"""/usr/share/common-licenses/GPL-3"""
OLD_NALA_SOURCES = Path(f"{ROOT}/etc/apt/sources.list.d/nala-sources.list")
"""/etc/apt/sources.list.d/nala-sources.list"""
NALA_SOURCES = Path(f"{ROOT}/etc/apt/sources.list.d/fetch.sources")
"""/etc/apt/sources.list.d/fetch.sources"""
NALA_DIR = Path(f"{ROOT}/var/lib/nala")
"""/var/lib/nala"""
NALA_LOGDIR = Path(f"{ROOT}/var/log/nala")
"""/var/log/nala"""
NALA_DEBUGLOG = NALA_LOGDIR / "nala-debug.log"
"""/var/log/nala/nala.debug.log"""
DPKG_LOG = NALA_LOGDIR / "dpkg-debug.log"
"""/var/log/nala/dpkg-debug.log"""
DPKG_STATUS_LOG = NALA_LOGDIR / "dpkg-status.log"
"""/var/log/nala/dpkg-status.log"""
NALA_TERM_LOG = Path(f"{ROOT}/var/log/nala/term.log")
"""/var/log/nala/term.log"""
NALA_HISTORY = Path(f"{ROOT}/var/lib/nala/history.json")
"""/var/lib/nala/history.json"""
PACSTALL_METADATA = Path(f"{ROOT}/var/log/pacstall/metadata")
"""/var/log/pacstall/metadata"""
REBOOT_REQUIRED = Path(f"{ROOT}/var/run/reboot-required")
"""/var/run/reboot-required"""
REBOOT_PKGS = Path(f"{ROOT}/var/run/reboot-required.pkgs")
"""'/var/run/reboot-required.pkgs'"""
NALA_LOCK_FILE = Path(f"{ROOT}/var/lock/nala.lock")
"""/var/lock/nala.lock"""

CACHE_DIR = Path(apt_pkg.config.find_dir("Dir::Cache", "/var/cache/apt"))
ETC_DIR = Path(apt_pkg.config.find_dir("Dir::Etc", "/etc/apt"))


def apt_file(key: str, prepend: Path, sub_path: str) -> Path:
	"""Get the apt directory. Build prepend/sub_path if None."""
	if file := apt_pkg.config.find_file(key):
		return Path(file)
	return prepend / sub_path


# Apt Directories
ARCHIVE_DIR = apt_file("Dir::Cache::Archives", CACHE_DIR, "/var/cache/apt/archives")
PARTIAL_DIR = ARCHIVE_DIR / "partial"
PKGCACHE = apt_file("Dir::Cache::pkgcache", CACHE_DIR, "pkgcache.bin")
SRCPKGCACHE = apt_file("Dir::Cache::srcpkgcache", CACHE_DIR, "srcpkgcache.bin")
LISTS_DIR = Path(apt_pkg.config.find_dir("Dir::State::Lists", "/var/lib/apt/lists/"))
LISTS_PARTIAL_DIR = LISTS_DIR / "partial"
DPKG_STATE = Path(apt_pkg.config.find_dir("Dir::State::status", "/var/lib/dpkg/status"))
SOURCELIST = apt_file("Dir::Etc::sourcelist", ETC_DIR, "/etc/apt/sources.list")
SOURCEPARTS = apt_file("Dir::Etc::sourceparts", ETC_DIR, "/etc/apt/sources.list.d")

# pylint: disable=invalid-name
HANDLER = Union[Callable[[int, Optional[FrameType]], Any], int, Handlers, None]

ERROR_PREFIX = color(_("Error:"), "RED")
WARNING_PREFIX = color(_("Warning:"), "YELLOW")
NOTICE_PREFIX = color(_("Notice:"), "YELLOW")

# Compiled Regex
ERRNO_PATTERN = re.compile(r"\[.*\]")


class SelectedState(IntEnum):
	"""Selected state constants for pkgs."""

	UNKNOWN = 0
	INSTALL = 1
	HOLD = 2
	DEINSTALL = 3
	PURGE = 4


class CurrentState(IntEnum):
	"""Current state constants for pkgs."""

	NOT_INSTALLED = 0
	UNPACKED = 1
	HALF_CONFIGURED = 2
	HALF_INSTALLED = 4
	CONFIG_FILES = 5
	INSTALLED = 6
	TRIGGERS_AWAITED = 7
	TRIGGERS_PENDING = 8


class InstState(IntEnum):
	"""Installed state constants for pkgs."""

	OK = 0
	REINSTREQ = 1
	HOLD_INST = 2
	HOLD_REINSTREQ = 3


# dpkg constants
NOTICES = (
	b"The currently running kernel version is not the expected kernel version",
	b"Please remove.",
	b"NOTICE:",
	b"Warning:",
)
SPAM = (
	# Stuff that's pretty useless
	"(Reading database",  #'(Reading database ... 247588 files and directories currently installed.)'
	"files and directories currently installed.)",
	"Selecting previously unselected package",  # 'Selecting previously unselected package chafa.'
	"Preparing to unpack",  # 'Preparing to unpack .../2-chafa_1.8.0-1_amd64.deb ...'
	"Extracting templates from packages:",
	"Fetched",
)
DPKG_STATUS = (
	b"Scanning processes...",
	b"Scanning candidates...",
	b"Scanning linux images...",
	b"Extracting templates from packages",
	b"Reading changelogs...",
	b"Retrieving bug reports...",
	b"Parsing Found/Fixed information...",
	b"[Working]",
	b"[Connecting",
	b"[Waiting for headers]",
	b"[Connected to",
)
DPKG_ERRORS = (
	b"dpkg:",
	b"failed to allocate memory for new entry",
	b"error processing package",
	b"error processing archive",
	b"Errors were encountered while processing",
	b"Processing was halted because there were too many errors",
)
CAT = r"""
   |\---/|
   | ,_, |
    \_`_/-..----.
 ___/ `   ' ,""+ \  sk
(__...'   __\    |`.___.';
  (_,...'(_,.`__)/'.....+
"""
