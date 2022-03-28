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
"""Nala fetch Module."""
from __future__ import annotations

import itertools
import re
import sys
from asyncio import Semaphore, gather, get_event_loop, run as aiorun
from ssl import SSLCertVerificationError
from subprocess import run
from typing import Union

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
from nala.options import arguments, parser
from nala.rich import fetch_progress
from nala.utils import ask, dprint, eprint

DEBIAN = "Debian"
UBUNTU = "Ubuntu"
DOMAIN_PATTERN = re.compile(r"https?://([A-Za-z_0-9.-]+).*")
UBUNTU_COUNTRY = re.compile(r"<mirror:countrycode>(.*)</mirror:countrycode>")
UBUNTU_MIRROR = re.compile(r"<link>(.*)</link>")
LIMITS = Limits(max_connections=100)
TIMEOUT = Timeout(timeout=5.0, read=1.0, pool=20.0)
ErrorTypes = Union[HTTPStatusError, HTTPError, SSLCertVerificationError, ReadTimeout]


class MirrorTest:
	"""Class to test mirrors."""

	def __init__(self, netselect: tuple[str, ...], release: str):
		"""Class to test mirrors."""
		self.netselect = netselect
		self.netselect_scored: list[str] = []
		self.release = release
		self.semp = Semaphore(1)
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
				tasks = (
					loop.create_task(self.net_select(mirror))
					for mirror in self.netselect
				)
				await gather(*tasks)

	async def net_select(self, mirror: str) -> None:
		"""Take a URL, ping the domain and score the latency."""
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
			res = str(int(response.elapsed.total_seconds() * 100))
			if arguments.sources:
				source_response = await self.client.get(
					f"{mirror}dists/{self.release}/main/source/Release"
				)
				source_response.raise_for_status()
			# We convert the float to integer in order to get rid of the decimal
			# From there we convert it to a string so we can prefix zeros for sorting

		except (HTTPError, SSLCertVerificationError) as error:
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
		else:
			print_error(error)

	if isinstance(error, ReadTimeout):
		debugger.append("Mirror too slow")


def ubuntu_mirror(country_list: tuple[str, ...] | None) -> tuple[str, ...]:
	"""Get and parse the Ubuntu mirror list."""
	print(_("Fetching Ubuntu mirrors..."))
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


def debian_mirror(country_list: tuple[str, ...] | None) -> tuple[str, ...]:
	"""Get and parse the Debian mirror list."""
	print(_("Fetching Debian mirrors..."))
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
	country_list: tuple[str, ...] | None,
	arches: tuple[str, ...],
) -> tuple[str, ...]:
	"""Parse the mirror."""
	mirror_set = set()
	if arguments.verbose:
		print(_("Parsing mirror list..."))
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


def detect_release() -> tuple[str | None, str | None]:
	"""Detect the distro and release."""
	if not arguments.debian and not arguments.ubuntu:
		return _lsb_release()

	if arguments.debian:
		return DEBIAN, arguments.debian
	return UBUNTU, arguments.ubuntu


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


def has_url(url: str) -> bool:
	"""Check if the mirror has the source for deb-src."""
	try:
		response = get(url)
		response.raise_for_status()
		return True
	except HTTPError:
		return False


def build_sources(
	release: str, component: str, sources: list[str], netselect_scored: tuple[str, ...]
) -> str:
	"""Build the sources file and return it as a string."""
	source = _("# Sources file built for nala") + "\n\n"
	num = 0
	for line in netselect_scored:
		# This splits off the score '030 http://mirror.steadfast.net/debian/'
		line = line[line.index("h") :]
		# This protects us from writing mirrors that we already have in the sources
		if any(line in mirror and release in mirror for mirror in sources):
			continue
		source += f"deb {line} {release} {component}\n"
		if arguments.sources:
			source += f"deb-src {line} {release} {component}\n"
		source += "\n"
		num += 1
		if num == arguments.fetches:
			break
	if num != arguments.fetches:
		eprint(
			_("{notice} We were unable to fetch {num} mirrors.").format(
				notice=NOTICE_PREFIX, num=arguments.fetches
			)
		)
	return source


def write_sources(source: str) -> None:
	"""Write mirrors to nala-sources.list."""
	with open(NALA_SOURCES, "w", encoding="utf-8") as file:
		file.write(source)
	print(_("Sources have been written to {file}").format(file=NALA_SOURCES))


def check_supported(
	distro: str | None, release: str | None, country_list: tuple[str, ...] | None
) -> tuple[tuple[str, ...], str]:
	"""Check if the distro is supported or not.

	If the distro is supported return mirror list and component.

	Error if the distro is not supported.
	"""
	if distro == DEBIAN and release != "n/a":
		component = "main" if arguments.foss else "main contrib non-free"
		return debian_mirror(country_list), component
	if distro in (UBUNTU, "Pop"):
		# It's ubuntu, you probably don't care about foss
		return ubuntu_mirror(country_list), "main restricted universe multiverse"
	if distro is None or release is None:
		eprint(
			_(
				"{error} There was an issue detecting release. "
				"You can specify manually\n"
			).format(error=ERROR_PREFIX)
		)
	else:
		eprint(
			_(
				"{error} {distro} {release} is unsupported.\n"
				"You can specify Ubuntu or Debian manually.\n"
			).format(error=ERROR_PREFIX, distro=distro, release=release)
		)
	parser.parse_args(["fetch", "--help"])
	sys.exit(1)


def fetch_checks() -> None:
	"""Perform checks and error if we shouldn't continue."""
	if NALA_SOURCES.exists():
		if not ask(
			_("{file} already exists.\n" "Continue and overwrite it?").format(
				file=color(str(NALA_SOURCES), "YELLOW")
			)
		):
			sys.exit(_("Abort."))

	elif not ask(
		_("The above mirrors will be written to {file}. Continue?").format(
			file=NALA_SOURCES
		)
	):
		sys.exit(_("Abort."))


def fetch() -> None:
	"""Fetch fast mirrors and write nala-sources.list."""
	# If supplied a country it needs to be a tuple
	country_list = (arguments.country,) if arguments.country else None

	distro, release = detect_release()
	netselect, component = check_supported(distro, release, country_list)
	assert distro and release

	dprint(netselect)
	dprint(f"Distro: {distro}, Release: {release}, Component: {component}")

	mirror_test = MirrorTest(netselect, release)
	aiorun(mirror_test.run_test())

	netselect_scored = mirror_test.get_scored()

	dprint(netselect_scored)
	dprint(f"Size of original list: {len(netselect)}")
	dprint(f"Size of scored list: {len(netselect_scored)}")
	dprint(f"Writing from: {netselect_scored[:arguments.fetches]}")
	source = build_sources(release, component, parse_sources(), netselect_scored)
	print(source, end="")
	fetch_checks()
	write_sources(source)
