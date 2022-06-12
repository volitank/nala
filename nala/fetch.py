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

import itertools
import re
import sys
from asyncio import Semaphore, gather, get_event_loop, run as aiorun
from ssl import SSLCertVerificationError, SSLError
from subprocess import run
from typing import Iterable, Optional, Union

import typer
from apt_pkg import get_architectures
from httpx import (
	AsyncClient,
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
from nala.options import ASSUME_YES, DEBUG, MAN_HELP, VERBOSE, arguments, nala
from nala.rich import ELLIPSIS, Live, Panel, Table, fetch_progress
from nala.utils import ask, dprint, eprint, sudo_check, term

DEBIAN = "Debian"
UBUNTU = "Ubuntu"
DOMAIN_PATTERN = re.compile(r"https?://([A-Za-z_0-9.-]+).*")
UBUNTU_COUNTRY = re.compile(r"<mirror:countrycode>(.*)</mirror:countrycode>")
UBUNTU_MIRROR = re.compile(r"<link>(.*)</link>")
LIMITS = Limits(max_connections=50)
TIMEOUT = Timeout(timeout=5.0, read=1.0, pool=20.0)
ErrorTypes = Union[HTTPStatusError, HTTPError, SSLError, ReadTimeout]

FETCH_HELP = _(
	"Nala will fetch mirrors with the lowest latency.\n\n"
	"For Debian https://mirror-master.debian.org/status/Mirrors.masterlist\n\n"
	"For Ubuntu https://launchpad.net/ubuntu/+archivemirrors-rss"
)


class MirrorTest:
	"""Class to test mirrors."""

	def __init__(self, netselect: tuple[str, ...], release: str, check_sources: bool):
		"""Class to test mirrors."""
		self.netselect = netselect
		self.netselect_scored: list[str] = []
		self.release = release
		self.sources = check_sources
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
				semp = Semaphore(50)
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

			domain = regex.group(1)
			debugger.append(f"Release Fetched: {domain}")
			await self.netping(mirror, debugger)
			self.progress.advance(self.task)

	async def netping(self, mirror: str, debugger: list[str]) -> bool:
		"""Fetch release file and score mirror."""
		try:
			response = await self.client.get(f"{mirror}dists/{self.release}/Release")
			response.raise_for_status()
			res = f"{int(response.elapsed.total_seconds() * 100)}"
			if self.sources:
				source_response = await self.client.get(
					f"{mirror}dists/{self.release}/main/source/Release"
				)
				source_response.raise_for_status()
			# We convert the float to integer in order to get rid of the decimal
			# From there we convert it to a string so we can prefix zeros for sorting

		except (HTTPError, SSLError) as error:
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
		self.mirror_list: list[str] = []
		self.user_list: list[str] = []
		self.index_list: tuple[int, ...]
		self._gen_mirror_list(release, sources, netselect_scored)

	def _gen_mirror_list(
		self, release: str, sources: Iterable[str], netselect_scored: Iterable[str]
	) -> None:
		"""Generate the mirror list for display."""
		for line in netselect_scored:
			url = line[line.index("h") :]
			if any(url in mirror and release in mirror for mirror in sources):
				continue
			self.mirror_list.append(line)
			if len(self.mirror_list) == self.count:
				break

	def clear(self, lines: int) -> None:
		"""Clear lines for the live display."""
		for _ in range(lines + self.errors):
			term.write(term.CURSER_UP + f"\r{' '*term.columns}\r".encode())
		self.errors = 0

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
			self.index_list = ask_index(self.count)
			if self.index_list:
				break
			self.errors += 1

	def final_mirrors(self) -> bool:
		"""Confirm that the final mirrors are okay."""
		self.live.start()
		self.live.update(
			Panel.fit(
				gen_table(self.user_list, no_index=True),
				title="[bold white] Selected Mirrors",
				title_align="left",
				border_style="bold green",
			),
			refresh=True,
		)
		self.live.stop()
		return ask(_("Are these mirrors okay?"), self)

	def set_user_list(self) -> None:
		"""Set the user selected list of mirrors."""
		self.user_list = [
			mirror
			for num, mirror in enumerate(self.mirror_list)
			if num in self.index_list
		]


def mirror_error(error: ErrorTypes, debugger: list[str]) -> None:
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
		else:
			print_error(error)

	if isinstance(error, ReadTimeout):
		debugger.append("Mirror too slow")


def ubuntu_mirror(country_list: Iterable[str] | None) -> tuple[str, ...]:
	"""Get and parse the Ubuntu mirror list."""
	print(_("Fetching Ubuntu mirrors") + ELLIPSIS)
	ubuntu = fetch_mirrors("https://launchpad.net/ubuntu/+archivemirrors-rss", "<item>")
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
	return parse_mirror(UBUNTU, ubuntu, country_list, tuple(get_architectures()))


def debian_mirror(country_list: Iterable[str] | None) -> tuple[str, ...]:
	"""Get and parse the Debian mirror list."""
	print(_("Fetching Debian mirrors") + ELLIPSIS)
	debian = fetch_mirrors(
		"https://mirror-master.debian.org/status/Mirrors.masterlist", "\n\n"
	)
	# This is what one of our "Mirrors might look like after split"
	# Site: mirrors.edge.kernel.org
	# Country: NL Netherlands
	# Country: US United States
	# Location: Amsterdam
	# Location: Parsippany, NJ
	# Location: San-Jose, CA
	# Archive-architecture: amd64 arm64 armel armhf i386 mips mips64el mipsel powerpc ppc64el s390x
	# Archive-http: /debian/
	# Sponsor: packet.net https://packet.net/
	return parse_mirror(DEBIAN, debian, country_list, tuple(get_architectures()))


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
		if country not in mirror:
			continue
		if distro == DEBIAN and (url := debian_parser(mirror, arches)):
			mirror_set.add(url)
			continue
		if distro == UBUNTU and (url := ubuntu_parser(mirror, arches)):
			mirror_set.add(url)
			continue
	return tuple(mirror_set)


def get_countries(master_mirror: tuple[str, ...]) -> tuple[str, ...]:
	"""Iterate the mirror list and return all valid countries."""
	country_list = set()
	# The way we split the information we get nice and pretty mirror selections
	for mirror in master_mirror:
		for line in mirror.splitlines():
			# Debian Countries
			if "Country:" in line:
				# Country: SE Sweden
				country_list.add(line.split()[1])
			# Ubuntu Countries
			elif "<mirror:countrycode>" in line:
				# <mirror:countrycode>US</mirror:countrycode>
				if result := re.search(UBUNTU_COUNTRY, line):
					country_list.add(result.group(1))
	return tuple(country_list)


def debian_parser(mirror: str, arches: tuple[str, ...]) -> str | None:
	"""Parse the Debian mirror."""
	url = "http://"
	if "Archive-http:" in mirror and all(arch in mirror for arch in arches):
		for line in mirror.splitlines():
			if line.startswith(("Archive-http:", "Site:")):
				# ['Site:', 'mirror.steadfastnet.com']
				# ['Archive-http:', '/debian/']
				url += line.split()[1]
	if url == "http://":
		return None
	return url


def ubuntu_parser(mirror: str, arches: tuple[str, ...]) -> str | None:
	"""Parse the Ubuntu mirror."""
	# First section we get from Ubuntu is garbage. Let's ditch it and get to business
	if "<title>Ubuntu Archive Mirrors Status</title>" in mirror:
		return None
	only_ports = "amd64" not in arches and "i386" not in arches
	for line in mirror.splitlines():
		# <link>http://mirror.steadfastnet.com/ubuntu/</link>
		if result := re.search(UBUNTU_MIRROR, line):
			if only_ports and "ubuntu-ports" not in result.group(1):
				return None
			return result.group(1)
	return None


def _lsb_release() -> tuple[str | None, str | None]:
	"""Run `lsb_release` and get the distro information."""
	lsb_id = None
	lsb_codename = None
	try:
		lsb_release = run(
			["lsb_release", "-idrc"], capture_output=True, check=True
		).stdout.decode()
	except OSError as error:
		dprint(error)
		return lsb_id, lsb_codename

	for line in lsb_release.splitlines():
		index = line.index("\t") + 1
		if "Distributor ID" in line:
			lsb_id = line[index:]
		if "Codename" in line:
			lsb_codename = line[index:]
	return lsb_id, lsb_codename


def detect_release(debian: str, ubuntu: str) -> tuple[str | None, str | None]:
	"""Detect the distro and release."""
	if not debian and not ubuntu:
		return _lsb_release()

	if debian:
		return DEBIAN, debian
	return UBUNTU, ubuntu


def parse_sources() -> list[str]:
	"""Read sources files on disk."""
	sources: list[str] = []
	for file in SOURCEPARTS.iterdir():
		if file != NALA_SOURCES:
			sources.extend(
				line
				for line in file.read_text(
					encoding="utf-8", errors="replace"
				).splitlines()
				if not line.startswith("#") and line
			)
	if SOURCELIST.exists():
		sources.extend(
			line
			for line in SOURCELIST.read_text(
				encoding="utf-8", errors="replace"
			).splitlines()
			if not line.startswith("#") and line
		)
	return sources


def gen_table(str_list: list[str], no_index: bool = False) -> Table:
	"""Generate table for the live display."""
	table = Table(padding=(0, 2), box=None)
	if not no_index:
		table.add_column("Index", justify="right", style="bold blue")
	table.add_column("Mirror")
	table.add_column("Latency", style="bold blue")
	for num, line in enumerate(str_list):
		latency, mirror = line.split()
		if no_index:
			table.add_row(mirror, f"{latency.lstrip('0')} ms")
			continue
		table.add_row(f"{num + 1}", mirror, f"{latency.lstrip('0')} ms")
	return table


def ask_index(count: int) -> tuple[int, ...]:
	"""Ask user about the mirrors they would like to use."""
	index_list: set[int] = set()
	response: list[str] | list[int]
	response = input(
		_("Mirrors you want to keep separated by spaces {selection}:").format(
			selection=f"({color('1')}..{color(str(count))})"
		)
		+ " "
	).split()

	if not response:
		response = list(range(count))

	for index in response:
		try:
			intdex = int(index) - 1
			if intdex not in range(count):
				term.write(term.CURSER_UP + f"\r{' '*term.columns}\r".encode())
				eprint(
					_("{error} Index {index} doesn't exist.").format(
						error=ERROR_PREFIX, index=color(index, "YELLOW")
					)
				)
				return ()
			index_list.add(intdex)
		except ValueError:
			term.write(term.CURSER_UP + f"\r{' '*term.columns}\r".encode())
			eprint(
				_("{error} Index {index} needs to be an integer.").format(
					error=ERROR_PREFIX, index=color(index, "YELLOW")
				)
			)
			return ()
	return tuple(index_list)


def build_sources(  # pylint: disable=too-many-arguments
	release: str,
	component: str,
	sources: list[str],
	netselect_scored: tuple[str, ...],
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
		if any(line in mirror and release in mirror for mirror in sources):
			continue
		source += f"deb {line} {release} {component}\n"
		if check_sources:
			source += f"deb-src {line} {release} {component}\n"
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
	foss: bool,
	ctx: typer.Context,
) -> tuple[tuple[str, ...], str]:
	"""Check if the distro is supported or not.

	If the distro is supported return mirror list and component.

	Error if the distro is not supported.
	"""
	if distro == DEBIAN and release != "n/a":
		component = "main" if foss else "main contrib non-free"
		return debian_mirror(country_list), component
	if distro in (UBUNTU, "Pop"):
		# It's ubuntu, you probably don't care about foss
		return ubuntu_mirror(country_list), "main restricted universe multiverse"
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
	fetches: int = typer.Option(
		0,
		help=_("Number of mirrors to fetch. [defaults: 16, --auto(3)]"),
		show_default=False,
	),
	sources: bool = typer.Option(
		False, "--sources", help=_("Add the source repos for the mirrors if it exists.")
	),
	foss: bool = typer.Option(
		False, "--foss", help=_("Omits contrib and non-free repos.")
	),
	auto: bool = typer.Option(
		False,
		"--auto",
		help=_("Run fetch uninteractively. Will still prompt for overwrite."),
	),
	debug: bool = DEBUG,
	assume_yes: bool = ASSUME_YES,
	country_list: Optional[list[str]] = typer.Option(
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

	distro, release = detect_release(debian, ubuntu)
	netselect, component = check_supported(distro, release, country_list, foss, ctx)
	assert distro and release

	dprint(netselect)
	dprint(f"Distro: {distro}, Release: {release}, Component: {component}")

	mirror_test = MirrorTest(netselect, release, sources)
	aiorun(mirror_test.run_test())

	netselect_scored = mirror_test.get_scored()

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
			fetch_live.clear(3)
			fetch_live.set_user_list()
			if fetch_live.final_mirrors():
				break
			fetch_live.clear(3)
			fetch_live.live.start()
		source = build_sources(
			release,
			component,
			sources_list,
			tuple(fetch_live.user_list),
			live=True,
			check_sources=sources,
		)
	write_sources(source)
