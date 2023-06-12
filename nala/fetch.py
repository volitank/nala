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
"""Nala fetch Module."""
from __future__ import annotations

import contextlib
import itertools
import re
import sys
from asyncio import Semaphore, gather, get_event_loop, run as aiorun
from pathlib import Path
from ssl import SSLCertVerificationError, SSLError
from typing import Iterable, List, Optional, Union

import typer
from apt import Cache
from apt_pkg import get_architectures
from httpx import (
	AsyncClient,
	ConnectError,
	ConnectTimeout,
	HTTPError,
	HTTPStatusError,
	Limits,
	ReadTimeout,
	Timeout,
	get,
)
from rich.progress import Progress, TaskID

from nala import _, color
from nala.constants import (
	ERROR_PREFIX,
	NALA_SOURCES,
	NOTICE_PREFIX,
	SOURCELIST,
	SOURCEPARTS,
)
from nala.downloader import print_error
from nala.error import ParserError
from nala.options import ASSUME_YES, DEBUG, MAN_HELP, VERBOSE, arguments, nala
from nala.rich import ELLIPSIS, Live, Panel, Table, fetch_progress
from nala.utils import ask, dprint, eprint, sudo_check, term

from debian.deb822 import Deb822  # isort:skip

DEBIAN = "Debian"
UBUNTU = "Ubuntu"
DEVUAN = "Devaun"

DOMAIN_PATTERN = re.compile(r"https?://([A-Za-z_0-9.-]+).*")
UBUNTU_COUNTRY = re.compile(r"<mirror:countrycode>(.*)</mirror:countrycode>")
UBUNTU_MIRROR = re.compile(r"<link>(.*)</link>")
FETCH_RANGE = re.compile(r"[0-9]+\.\.[0-9]+")
LIMITS = Limits(max_connections=50)
TIMEOUT = Timeout(timeout=5.0, read=1.0, pool=20.0)
ErrorTypes = Union[HTTPStatusError, HTTPError, SSLError, ReadTimeout, OSError]
INVALID_FILENAME_CHARS = re.compile("[^a-zA-Z0-9_.-]", re.ASCII)

FETCH_HELP = _(
	"Nala will fetch mirrors with the lowest latency.\n\n"
	"For Debian https://mirror-master.debian.org/status/Mirrors.masterlist\n\n"
	"For Ubuntu https://launchpad.net/ubuntu/+archivemirrors-rss"
)

# pylint: disable=too-many-instance-attributes
class MirrorTest:
	"""Class to test mirrors."""

	def __init__(
		self,
		netselect: tuple[str, ...],
		release: str,
		check_sources: bool,
		https_only: bool,
	):
		"""Class to test mirrors."""
		self.netselect = netselect
		self.netselect_scored: list[str] = []
		self.release = release
		self.sources = check_sources
		self.https_only = https_only
		self.client: AsyncClient
		self.progress: Progress
		self.task: TaskID

	async def run_test(self) -> None:
		"""Test mirrors."""
		with fetch_progress as self.progress:
			self.task = self.progress.add_task("", total=len(self.netselect))
			async with AsyncClient(
				follow_redirects=True, limits=LIMITS, timeout=TIMEOUT
			) as self.client:
				loop = get_event_loop()
				semp = Semaphore(25)
				tasks = [
					loop.create_task(self.net_select(mirror, semp))
					for mirror in self.netselect
				]
				await gather(*tasks)

	async def net_select(self, mirror: str, semp: Semaphore) -> None:
		"""Take a URL, ping the domain and score the latency."""
		async with semp:
			debugger = [f"Current Mirror: {mirror}"]

			regex = re.search(DOMAIN_PATTERN, mirror)
			if not regex:
				self.progress.advance(self.task)
				debugger.append("Regex Failed")
				dprint(debugger)
				return

			domain = regex[1]
			debugger.append(f"Regex Match: {domain}")
			with contextlib.suppress(RuntimeError):
				await self.netping(mirror, debugger)
			self.progress.advance(self.task)

	async def netping(self, mirror: str, debugger: list[str]) -> bool:
		"""Fetch release file and score mirror."""
		secure = False
		try:
			# Try to do https first
			https = mirror.replace("http://", "https://")
			try:
				response = await self.client.get(f"{https}dists/{self.release}/Release")
				response.raise_for_status()
				secure = True
				mirror = https
			# We catch all Exceptions because we will fall back to http
			except Exception as error:  # pylint: disable=broad-except
				debugger.append(f"https attempt failed: {error}")
				if self.https_only:
					mirror_error(error, debugger)
					dprint(debugger)
					return False

			# We can fall back to http if it's necessary
			if not secure:
				response = await self.client.get(
					f"{mirror}dists/{self.release}/Release"
				)
				response.raise_for_status()

			# Get rid of the decimal so we can prefix zeros for sorting.
			res = f"{response.elapsed.total_seconds() * 100:.0f}"
			if self.sources:
				source_response = await self.client.get(
					f"{mirror}dists/{self.release}/main/source/Release"
				)
				source_response.raise_for_status()

		# We catch all exceptions here because it really doesn't matter
		except Exception as error:  # pylint: disable=broad-except
			mirror_error(error, debugger)
			dprint(debugger)
			return False

		debugger.append(f"Download ms: {res}")
		if len(res) == 2:
			res = f"0{res}"
		elif len(res) == 1:
			res = f"00{res}"
		elif len(res) > 3:
			debugger.append("Mirror too slow")
			dprint(debugger)
			return False

		debugger.append(f"Appended: {res} {mirror}")
		dprint(debugger)
		self.netselect_scored.append(f"{res} {mirror}")
		return True

	def get_scored(self) -> tuple[str, ...]:
		"""Return sorted tuple."""
		return tuple(sorted(self.netselect_scored))


class FetchLive:
	"""Interactive Fetch."""

	def __init__(  # pylint: disable=too-many-arguments
		self,
		live: Live,
		release: str,
		sources: list[str],
		count: int,
		netselect_scored: tuple[str, ...],
	) -> None:
		"""Interactive Fetch."""
		self.live = live
		self.errors = 0
		self.count = count
		self.mirror_list: dict[int, str] = {}
		self.user_list: dict[int, str] = {}
		self.index_list: set[int]
		self._gen_mirror_list(release, sources, netselect_scored)

	def _gen_mirror_list(
		self, release: str, sources: Iterable[str], netselect_scored: Iterable[str]
	) -> None:
		"""Generate the mirror list for display."""
		index = 0
		for line in netselect_scored:
			url = line[line.index(":") :].rstrip("/")
			if any(url in mirror and release in mirror for mirror in sources):
				continue

			self.mirror_list[index] = line
			index += 1

			if len(self.mirror_list) == self.count:
				break

	def clear(self, lines: int) -> None:
		"""Clear lines for the live display."""
		for _ in range(lines + self.errors):
			term.write(term.CURSER_UP + f"\r{' '*term.columns}\r".encode())
		self.errors = 0

	def debug(self, msg: object) -> None:
		"""Display debugging information with the live display."""
		if arguments.debug:
			term.write(term.CURSER_UP * 24 + f"\r{' ' * term.columns}\r".encode())
			term.write(term.CURSER_UP + f"\r{' ' * term.columns}\r".encode())
			term.write(term.CURSER_UP + f"\r{' ' * term.columns}\r".encode())
			term.write(term.CURSER_UP + f"\r{' ' * term.columns}\r".encode())
			term.write(f"{msg}".encode())
			term.write(term.CURSER_DOWN * 26 + f"\r{' ' * term.columns}\r".encode())

	def error(self, msg: object) -> None:
		"""Print an error out and keep track of how many."""
		self.errors += 1
		term.write(term.CURSER_UP + f"\r{' ' * term.columns}\r".encode())
		eprint(msg)

	def choose_mirrors(self) -> None:
		"""Allow user to choose their mirrors."""
		while True:
			self.live.update(
				Panel.fit(
					gen_table(self.mirror_list),
					title="[bold default] Fastest Mirrors",
					title_align="left",
					border_style="bold green",
				),
				refresh=True,
			)
			self.live.stop()
			self.index_list = self.ask_index(self.count)
			if self.index_list:
				break

	def final_mirrors(self) -> bool:
		"""Confirm that the final mirrors are okay."""
		self.live.start()
		self.live.update(
			Panel.fit(
				gen_table(self.user_list),
				title="[bold white] Selected Mirrors",
				title_align="left",
				border_style="bold green",
			),
			refresh=True,
		)
		self.live.stop()
		return ask(_("Are these mirrors okay?"))

	def set_user_list(self) -> None:
		"""Set the user selected list of mirrors."""
		self.user_list = {
			num: mirror
			for num, mirror in self.mirror_list.items()
			if num in self.index_list
		}

	def ask_index(self, count: int) -> set[int]:
		"""Ask user about the mirrors they would like to use."""
		index_list: set[int] = set()
		response: str = input(
			_(
				"Mirrors you want to keep, separated by space or comma {selection}:"
			).format(selection=f"({color('1')}..{color(str(count))})")
			+ " "
		)

		# Small single use wrapper to just clean up the code a bit
		# Debug has to be done at the return or everything will get messed up
		def _debug(passthrough: set[int]) -> set[int]:
			self.debug(
				f"Response: {response or 'Default'}\n"
				f"Range: {range(1, count + 1)}\n"
				f"User Range: {passthrough}"
			)
			return passthrough

		try:
			for index in range_from_str(response, count):
				# Plus one is for taking care of the user seeing
				# numbers starting at 1 instead of 0
				if index not in range(1, count + 1):
					self.error(
						_("{error} Index {index} doesn't exist.").format(
							error=ERROR_PREFIX, index=color(index, "YELLOW")
						)
					)
					# Returning an empty set will restart the self.choose_mirrors loop
					return _debug(set())
				index_list.add(index - 1)

		except ValueError as error:
			self.error(
				_("{error} {value_error}").format(
					error=ERROR_PREFIX,
					value_error=color(f"{error}".capitalize(), "YELLOW"),
				)
			)
			# Returning an empty set will restart the self.choose_mirrors loop
			return _debug(set())

		except ParserError as error:
			self.error(
				_("{error} Parser: {parser}").format(error=ERROR_PREFIX, parser=error)
			)
		return _debug(index_list)


def mirror_error(error: Exception, debugger: list[str]) -> None:
	"""Handle errors when mirror testing."""
	if isinstance(error, HTTPStatusError):
		if arguments.verbose:
			print_error(error)
		debugger.append(f"Status Code: {error.response.status_code}")
		return

	if arguments.verbose:
		if isinstance(error, SSLCertVerificationError):
			eprint(f"{ERROR_PREFIX} {error.reason} {error.verify_message}")
		elif isinstance(error, SSLError):
			eprint(f"{ERROR_PREFIX} {error.reason}")
		elif isinstance(error, (ConnectError, ConnectTimeout)):
			print_error(error)
		else:
			eprint(f"{ERROR_PREFIX} {error}")

	if isinstance(error, ReadTimeout):
		debugger.append("Mirror too slow")


def get_and_parse_mirror(
	distro: str, country_list: Iterable[str] | None
) -> tuple[str, ...]:
	"""Get and parse the mirror list."""
	print(_("Fetching {distro} mirrors").format(distro=distro) + ELLIPSIS)
	if distro == DEBIAN:
		mirror = fetch_mirrors(
			"https://mirror-master.debian.org/status/Mirrors.masterlist", "\n\n"
		)
		# This is what one of our "Mirrors might look like after split"
		# Site: mirrors.edge.kernel.org
		# Country: NL Netherlands
		# Country: US United States
		# Location: Amsterdam
		# Location: Parsippany, NJ
		# Location: San-Jose, CA
		# Archive-architecture: amd64 arm64 armel armhf i386
		# Archive-http: /debian/
		# Sponsor: packet.net https://packet.net/
	elif distro == UBUNTU:
		mirror = fetch_mirrors(
			"https://launchpad.net/ubuntu/+archivemirrors-rss", "<item>"
		)
		# This is what one of our "Mirrors might look like after split"
		#      <title>Steadfast Networks</title>
		#      <link>http://mirror.steadfastnet.com/ubuntu/</link>
		#      <description>
		#      </description>
		#      <mirror:bandwidth>80</mirror:bandwidth>
		#      <mirror:location>
		#        <mirror:continent>North America</mirror:continent>
		#        <mirror:country>United States</mirror:country>
		#        <mirror:countrycode>US</mirror:countrycode>
		#      </mirror:location>
		#      <pubDate>Fri, 24 Dec 2021 05:26:30 -0000</pubDate>
		#      <guid>http://mirror.steadfastnet.com/ubuntu/</guid>
		#    </item>
	elif distro == DEVUAN:
		mirror = fetch_mirrors("https://pkgmaster.devuan.org/mirror_list.txt", "\n\n")
		# FQDN:  sledjhamr.org
		# BaseURL:  sledjhamr.org/devuan
		# Bandwidth:  1Gb/s
		# Rate:  30min
		# Country:  Netherlands
		# CountryCode:  NL | BE | CH | CZ | DE | DK | FR | GB | GG | IE | IM | JE | LU
		# Protocols:  HTTP | HTTPS | FTP | RSYNC
		# Active:  yes
		# DNSRR:  yes
		# DNSRRCC:  yes
	else:
		# We should never really hit this.
		sys.exit(
			_("{error} Internal Error. Distro detection must be broken").format(
				error=ERROR_PREFIX
			)
		)
	return parse_mirror(distro, mirror, country_list, tuple(get_architectures()))


def fetch_mirrors(url: str, splitter: str) -> tuple[str, ...]:
	"""Attempt to fetch the url and split a list based on the splitter."""
	try:
		response = get(url, timeout=15, follow_redirects=True)
		response.raise_for_status()
		mirror_list = response.text.split(splitter)
	except HTTPError:
		sys.exit(
			_("{error} unable to connect to {mirror}").format(
				error=ERROR_PREFIX, mirror=url
			)
		)
	return tuple(mirror_list)


def parse_mirror(
	distro: str,
	master_mirror: tuple[str, ...],
	country_list: Iterable[str] | None,
	arches: tuple[str, ...],
) -> tuple[str, ...]:
	"""Parse the mirror."""
	mirror_set = set()
	if arguments.verbose:
		print(_("Parsing mirror list") + ELLIPSIS)
	# If no country is supplied then our list will be all countries
	countries = country_list or get_countries(master_mirror)
	for country, mirror in itertools.product(countries, master_mirror):
		if (
			distro == DEBIAN
			and f"Country: {country.upper()}" in mirror
			and (url := debian_parser(mirror, arches))
		):
			mirror_set.add(url)
			continue

		if (
			distro == UBUNTU
			and f"<mirror:countrycode>{country.upper()}</mirror:countrycode>" in mirror
			and (url := ubuntu_parser(mirror, arches))
		):
			mirror_set.add(url)
			continue

		if distro == DEVUAN:
			for line in mirror.splitlines():
				# CountryCode:  NL | BE | CH
				if line.startswith("CountryCode:") and country.upper() in line:
					if url := devuan_parser(mirror):
						mirror_set.add(url)
						continue

	return tuple(mirror_set)


def get_countries(master_mirror: tuple[str, ...]) -> tuple[str, ...]:
	"""Iterate the mirror list and return all valid countries."""
	country_list = set()
	# The way we split the information we get nice and pretty mirror selections
	for mirror in master_mirror:
		for line in mirror.splitlines():
			# Devuan Countries
			if "CountryCode:" in line:
				# CountryCode:  BG | GR | RO | MK | RS | TR
				for country in line.split()[1:]:
					if line == "|":
						continue
					country_list.add(country)
			# Debian Countries
			elif "Country:" in line:
				# Country: SE Sweden
				country_list.add(line.split()[1])
			# Ubuntu Countries
			elif "<mirror:countrycode>" in line:
				# <mirror:countrycode>US</mirror:countrycode>
				if result := re.search(UBUNTU_COUNTRY, line):
					country_list.add(result[1])
	return tuple(country_list)


def devuan_parser(mirror: str) -> str | None:
	"""Parse the Debuan mirror."""
	if "HTTP" not in mirror:
		return None
	url = None
	for line in mirror.splitlines():
		# BaseURL:  sledjhamr.org/devuan
		if line.startswith("BaseURL:"):
			url = line.split()[1]

	return f"http://{url}/devuan/" if url else None


def debian_parser(mirror: str, arches: tuple[str, ...]) -> str | None:
	"""Parse the Debian mirror."""
	url = "http://"
	if "Archive-http:" in mirror and all(arch in mirror for arch in arches):
		for line in mirror.splitlines():
			if line.startswith(("Archive-http:", "Site:")):
				# ['Site:', 'mirror.steadfastnet.com']
				# ['Archive-http:', '/debian/']
				url += line.split()[1]

	return None if url == "http://" else url


def ubuntu_parser(mirror: str, arches: tuple[str, ...]) -> str | None:
	"""Parse the Ubuntu mirror."""
	# First section we get from Ubuntu is garbage. Let's ditch it and get to business
	if "<title>Ubuntu Archive Mirrors Status</title>" in mirror:
		return None
	only_ports = "amd64" not in arches and "i386" not in arches
	for line in mirror.splitlines():
		# <link>http://mirror.steadfastnet.com/ubuntu/</link>
		if result := re.search(UBUNTU_MIRROR, line):
			return None if only_ports and "ubuntu-ports" not in result[1] else result[1]
	return None


def detect_release(
	debian: str, ubuntu: str, devuan: str
) -> tuple[str | None, str | None]:
	"""Detect the distro and release."""
	# Check if the release was specified.
	for dist, switch in (
		(DEBIAN, debian),
		(DEVUAN, devuan),
		(UBUNTU, ubuntu),
	):
		if switch:
			return dist, switch

	# If no release is specified try to detect it by keyrings.
	cache = Cache()
	for keyring in (
		"devuan-keyring",
		"debian-archive-keyring",
		"ubuntu-keyring",
		"apt",
	):
		if (
			keyring not in cache
			or not (cand := cache[keyring].candidate)
			or not (origin := cand.origins)
		):
			continue
		return origin[0].origin, origin[0].codename

	# Something is very wrong if apt has no origin.
	# So we parse os-release to see if we can detect anything
	release_file = Path("/etc/os-release")
	if not release_file.is_file():
		# This will throw an error at the next step
		# ERROR: There was an issue detecting release.
		return None, None

	os_release: dict[str, str] = {}
	for line in release_file.read_text(encoding="utf-8", errors="replace").splitlines():
		entry = line.split("=")
		os_release[entry[0]] = entry[1].strip('"')

	# If there is no name we'll just have it throw an error
	if not (name := os_release.get("NAME")):
		return None, None

	# This block is for Debian Testing/Sid. As they don't have a codename key.
	release = os_release.get("DEBIAN_CODENAME") or os_release.get("UBUNTU_CODENAME")
	if not release and "Debian" in name:
		try:
			release = os_release["PRETTY_NAME"].split().pop()
		except IndexError:
			return name, "Unknown"
	return name, release


def parse_sources() -> list[str]:
	"""Read sources files on disk."""
	sources: list[str] = []
	for file in [*SOURCEPARTS.iterdir(), SOURCELIST]:
		if (
			file == NALA_SOURCES
			or not file.is_file()
			or INVALID_FILENAME_CHARS.search(file.name)
		):
			continue
		if file.parent == SOURCEPARTS and file.suffix not in [".list", ".sources"]:
			continue
		if file.suffix in ".sources":
			sources.extend(
				f"{deb} {uri} {suite}"
				for deb822 in Deb822.iter_paragraphs(
					file.read_text(encoding="utf-8", errors="replace")
				)
				for deb in deb822.get("Types", "").split()
				for uri in deb822.get("URIs", "").split()
				for suite in deb822.get("Suites", "").split()
				for enabled in [deb822.get("Enabled", "yes").lower()]
				if enabled not in ["no", "false"]
				and any(digit not in "0" for digit in enabled)
			)

		else:
			sources.extend(
				line
				for line in file.read_text(
					encoding="utf-8", errors="replace"
				).splitlines()
				if not line.lstrip().startswith("#") and line
			)
	return sources


def gen_table(str_list: dict[int, str]) -> Table:
	"""Generate table for the live display."""
	master_table = Table(padding=(0, 0), box=None)
	table = Table(padding=(0, 2), box=None)

	table.add_column("Index", justify="right", style="bold blue")
	table.add_column("Mirror")
	table.add_column("Score", style="bold blue")

	for num, line in str_list.items():
		latency, mirror = line.split()
		table.add_row(f"{num + 1}", mirror, f"{latency.lstrip('0')} ms")

	master_table.add_row(table)
	master_table.add_row(
		# Add in a new line and indentation to line up the text
		"\n  "
		+ _("Score is how many milliseconds it takes to download the Release file"),
		style="italic",
	)
	return master_table


def range_from_str(string: str, count: int) -> Iterable[int]:
	"""Get a range of integers from a string.

	See live.ask_index()
	"""
	# If it's empty the default is assumed
	if not string:
		# We return plus 1 for everything to match the numbers
		# That the user sees
		return list(range(1, count + 1))

	if match := FETCH_RANGE.search(string):
		resp = match.string.split("..")
		# This will mean someone is trying to get a range
		# 0..10 = 0, 1, 2, 3, and so forth
		if len(resp) == 2:
			start = int(resp[0])
			stop = int(resp[1]) + 1
			# Convert the strings from input into integers and create the range
			return range(start, stop)

		# They must be trying to get a range of even or odd
		# 0..0..10 = 0 2 4 6 8 10
		if (step := int(resp[0])) not in {0, 1}:
			raise ParserError("0 for even and 1 for odd")

		start = int(resp[1])
		stop = int(resp[2]) + 1

		is_odds = step == 1
		return {
			num
			for num in range(start, stop)
			if (is_odds and num % 2) or (not is_odds and not num % 2)
		}

	# Use a set as we don't want to have duplicate numbers
	return {int(num) for num in re.split(r",|\s", string) if num}


def format_component(url: str, component: str, release: str, non_free: bool) -> str:
	"""Add non-free-firmware repository if applicable."""
	# Starting with bookworm there is an additional component, non-free-firmware.
	# The best way to do this is just check if it exists for the mirror.
	if not non_free:
		return component
	try:
		get(
			f"{url}/dists/{release}/non-free-firmware/",
			timeout=15,
			follow_redirects=True,
		).raise_for_status()
	except HTTPError:
		return component
	return f"{component} non-free-firmware"


def build_sources(  # pylint: disable=too-many-arguments
	release: str,
	component: str,
	sources: list[str],
	netselect_scored: Iterable[str],
	non_free: bool,
	fetches: int = 3,
	live: bool = False,
	check_sources: bool = False,
) -> str:
	"""Build the sources file and return it as a string."""
	source = "# Sources file built for nala\n\n"
	num = 0
	for line in netselect_scored:
		# This splits off the score '030 http://mirror.steadfast.net/debian/'
		line = line[line.index("h") :]
		# This protects us from writing mirrors that we already have in the sources
		if any(line.rstrip("/") in mirror and release in mirror for mirror in sources):
			continue

		deb_entry = (
			f"{line} {release} {format_component(line, component, release, non_free)}"
		)
		source += f"deb {deb_entry}\n"
		if check_sources:
			source += f"deb-src {deb_entry}\n"
		source += "\n"
		num += 1
		if not live and num == fetches:
			break
	if not live and num != fetches:
		eprint(
			_("{notice} Nala was unable to fetch {num} mirrors.").format(
				notice=NOTICE_PREFIX, num=fetches
			)
		)
	return source


def write_sources(source: str) -> None:
	"""Write mirrors to nala-sources.list."""
	with open(NALA_SOURCES, "w", encoding="utf-8") as file:
		file.write(source)
	print(_("Sources have been written to {file}").format(file=NALA_SOURCES))


def check_supported(
	distro: str | None,
	release: str | None,
	country_list: Iterable[str] | None,
	non_free: bool,
	ctx: typer.Context,
) -> tuple[tuple[str, ...], str]:
	"""Check if the distro is supported or not.

	If the distro is supported return mirror list and component.

	Error if the distro is not supported.
	"""
	if distro and release:
		if distro in (DEBIAN, DEVUAN) and release != "n/a":
			component = "main contrib non-free" if non_free else "main"
			return get_and_parse_mirror(distro, country_list), component
		if distro == UBUNTU:
			# It's ubuntu, you probably don't care about foss
			return (
				get_and_parse_mirror(distro, country_list),
				"main restricted universe multiverse",
			)

	if distro is None or release is None:
		eprint(
			_("{error} There was an issue detecting release.").format(
				error=ERROR_PREFIX
			),
			end="\n\n",
		)
	else:
		eprint(
			_("{error} {distro} {release} is unsupported.").format(
				error=ERROR_PREFIX, distro=distro, release=release
			)
		)
	eprint(_("You can specify Ubuntu or Debian manually."), end="\n\n")
	eprint(ctx.get_help())
	sys.exit(1)


def fetch_checks(source: str) -> None:
	"""Perform checks and error if we shouldn't continue."""
	print(source, end="")
	if NALA_SOURCES.exists():
		if not ask(
			_("{file} already exists.\nContinue and overwrite it?").format(
				file=color(NALA_SOURCES, "YELLOW")
			)
		):
			sys.exit(_("Abort."))

	elif not ask(
		_("The above mirrors will be written to {file}. Continue?").format(
			file=NALA_SOURCES
		)
	):
		sys.exit(_("Abort."))


@nala.command(
	short_help=_("Fetch fast mirrors to speed up downloads."), help=FETCH_HELP
)
# pylint: disable=unused-argument,too-many-arguments,too-many-locals
def fetch(
	ctx: typer.Context,
	debian: str = typer.Option("", metavar="sid", help=_("Choose the Debian release.")),
	ubuntu: str = typer.Option(
		"", metavar="jammy", help=_("Choose the Ubuntu release.")
	),
	devuan: str = typer.Option(
		"", metavar="stable", help=_("Choose the Devuan release.")
	),
	fetches: int = typer.Option(
		0,
		help=_("Number of mirrors to fetch. [defaults: 16, --auto(3)]"),
		show_default=False,
	),
	https_only: bool = typer.Option(
		False, "--https-only", help="Only get https mirrors."
	),
	sources: bool = typer.Option(
		False, "--sources", help=_("Add the source repos for the mirrors if it exists.")
	),
	non_free: bool = typer.Option(
		False, "--non-free", help=_("Add contrib and non-free repos.")
	),
	auto: bool = typer.Option(
		False,
		"--auto",
		help=_("Run fetch uninteractively. Will still prompt for overwrite."),
	),
	debug: bool = DEBUG,
	assume_yes: bool = ASSUME_YES,
	country_list: Optional[List[str]] = typer.Option(
		None,
		"-c",
		"--country",
		metavar="US",
		help=_("Choose only mirrors of a specific ISO country code."),
	),
	verbose: bool = VERBOSE,
	man_help: bool = MAN_HELP,
) -> None:
	"""Nala will fetch mirrors with the lowest latency.

	For Debian https://mirror-master.debian.org/status/Mirrors.masterlist

	For Ubuntu https://launchpad.net/ubuntu/+archivemirrors-rss
	"""
	sudo_check()

	# Set dynamic default fetch option
	if fetches == 0:
		fetches = 3 if auto else 16

	distro, release = detect_release(debian, ubuntu, devuan)
	netselect, component = check_supported(distro, release, country_list, non_free, ctx)
	assert distro and release

	dprint(netselect)
	dprint(f"Distro: {distro}, Release: {release}, Component: {component}")

	mirror_test = MirrorTest(netselect, release, sources, https_only)
	aiorun(mirror_test.run_test())

	if not (netselect_scored := mirror_test.get_scored()):
		sys.exit(
			_("{error} Nala was unable to find any mirrors.").format(error=ERROR_PREFIX)
		)

	dprint(netselect_scored)
	dprint(f"Size of original list: {len(netselect)}")
	dprint(f"Size of scored list: {len(netselect_scored)}")
	dprint(f"Writing from: {netselect_scored[:fetches]}")

	if auto:
		source = build_sources(
			release,
			component,
			parse_sources(),
			netselect_scored,
			non_free,
			fetches=fetches,
			check_sources=sources,
		)

		fetch_checks(source)
		write_sources(source)
		return

	sources_list = parse_sources()
	with Live(auto_refresh=False) as live:
		fetch_live = FetchLive(live, release, sources_list, fetches, netselect_scored)
		while True:
			fetch_live.choose_mirrors()
			fetch_live.clear(2)
			fetch_live.set_user_list()
			if fetch_live.final_mirrors():
				break
			fetch_live.clear(2)
			fetch_live.live.start()
		source = build_sources(
			release,
			component,
			sources_list,
			fetch_live.user_list.values(),
			non_free,
			live=True,
			check_sources=sources,
		)
	write_sources(source)
