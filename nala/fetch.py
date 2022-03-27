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
from asyncio import (
	Semaphore,
	create_subprocess_exec,
	gather,
	get_event_loop,
	run as aiorun,
)
from asyncio.streams import StreamReader
from asyncio.subprocess import PIPE, Process
from subprocess import run
from typing import cast

from apt_pkg import get_architectures
from httpx import HTTPError, get
from rich.progress import Progress, TaskID

from nala import _, color
from nala.constants import (
	ERROR_PREFIX,
	NALA_SOURCES,
	NOTICE_PREFIX,
	SOURCELIST,
	SOURCEPARTS,
)
from nala.options import arguments, parser
from nala.rich import fetch_progress
from nala.utils import ask, dprint, eprint, vprint

netselect_scored = []

DEBIAN = "Debian"
UBUNTU = "Ubuntu"
DOMAIN_PATTERN = re.compile(r"https?://([A-Za-z_0-9.-]+).*")
UBUNTU_COUNTRY = re.compile(r"<mirror:countrycode>(.*)</mirror:countrycode>")
UBUNTU_MIRROR = re.compile(r"<link>(.*)</link>")


async def ping_output(proc: Process, mirror: str) -> list[str] | bool:
	"""Read the output of the ping process."""
	lines: list[str] = []
	no_answer = 0
	while True:
		line = await cast(StreamReader, proc.stdout).readline()
		if line == b"":
			break
		if b"no answer yet" in line:
			no_answer += 1
			if no_answer == 2:
				proc.terminate()
				ping_error("", mirror)
				return False
		lines.append(line.decode("utf-8").rstrip())
	return lines


async def ping(mirror: str) -> float | bool:
	"""Ping 5 times and return the result."""
	proc = await create_subprocess_exec(
		"ping", "-c", "5", "-W", "1", "-O", mirror, stdout=PIPE, stderr=PIPE
	)

	lines = await ping_output(proc, mirror)
	if isinstance(lines, bool):
		return False

	if await proc.wait() != 0:
		error = await cast(StreamReader, proc.stderr).read()
		ping_error(error.decode("utf-8").strip(), mirror)
		return False

	stats = ""
	for line in lines:
		if line.startswith("rtt"):
			stats = line
	# 'rtt min/avg/max/mdev = 37.701/43.665/52.299/4.887 ms'
	return round(float(stats.split(" = ")[1].split("/")[1]))


async def net_select(
	mirror: str, task: TaskID, progress: Progress, semp: Semaphore
) -> None:
	"""Take a URL, ping the domain and score the latency."""
	async with semp:
		debugger = [f"Current Mirror: {mirror}"]
		# Regex to get the domain
		regex = re.search(DOMAIN_PATTERN, mirror)
		if not regex:
			progress.advance(task)
			debugger.append("Regex Failed")
			dprint(debugger)
			return
		domain = regex.group(1)
		debugger.append(f"Pinged: {domain}")
		await netping(domain, mirror, debugger)
		progress.advance(task)


async def netping(domain: str, mirror: str, debugger: list[str]) -> bool:
	"""Ping the domain and score it."""
	# We convert the float to integer in order to get rid of the decimal
	# From there we convert it to a string so we can prefix zeros for sorting
	if not (res := str(await ping(domain))):
		debugger.append("Ping ms: Packet Loss")
		dprint(debugger)
		return False

	debugger.append(f"Ping ms: {res}")
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
	netselect_scored.append(f"{res} {mirror}")
	return True


def ping_error(error: str, mirror: str) -> None:
	"""Handle error on ping."""
	if arguments.verbose:
		if not error:
			eprint(
				_("{error} Packets were lost: {mirror}").format(
					error=ERROR_PREFIX, mirror=mirror
				)
			)
			return
		if "Temporary failure in name resolution" in error:
			eprint(
				_("{error} Temporary failure in name resolution: {mirror}").format(
					error=ERROR_PREFIX, mirror=mirror
				)
			)
			return
		eprint(f"{color(error, 'RED')}")


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
		if "Distributor ID" in line:
			index = line.index("\t") + 1
			lsb_id = line[index:]
		if "Codename" in line:
			index = line.index("\t") + 1
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


def write_sources(release: str, component: str, sources: list[str]) -> None:
	"""Write mirrors to nala-sources.list."""
	with open(NALA_SOURCES, "w", encoding="utf-8") as file:
		writing = color(_("Writing:"), "GREEN")
		print(f"{writing} {NALA_SOURCES}\n")
		print(_("# Sources file built for nala"), file=file, end="\n\n")
		num = 0
		for line in netselect_scored:
			# This splits off the score '030 http://mirror.steadfast.net/debian/'
			line = line[line.index("h") :]
			# This protects us from writing mirrors that we already have in the sources
			if any(line in mirror and release in mirror for mirror in sources):
				continue
			if not has_url(f"{line}dists/{release}/Release"):
				vprint(
					_("{notice} {url} does not have the release {release}\n").format(
						notice=NOTICE_PREFIX, url=line, release=color(release, "YELLOW")
					)
				)
				continue
			source = f"deb {line} {release} {component}\n"
			if arguments.sources and has_url(
				f"{line}dists/{release}/main/source/Release"
			):
				source += f"deb-src {line} {release} {component}\n"
			print(source)
			print(source, file=file)
			num += 1
			if num == arguments.fetches:
				break
		if num != arguments.fetches:
			eprint(
				_("{notice} We were unable to fetch {num} mirrors.").format(
					notice=NOTICE_PREFIX, num=arguments.fetches
				)
			)


async def test_mirrors(netselect: tuple[str, ...]) -> None:
	"""Test mirrors."""
	semp = Semaphore(256)
	with fetch_progress as progress:
		total = len(netselect)
		task = progress.add_task("", total=total)
		loop = get_event_loop()
		tasks = (
			loop.create_task(net_select(mirror, task, progress, semp))
			for mirror in netselect
		)
		await gather(*tasks)


def check_supported(
	distro: str | None, release: str | None, country_list: tuple[str, ...] | None
) -> tuple[tuple[str, ...], str]:
	"""Check if the distro is supported or not.

	If the distro is supported return mirror list and component.

	Error if the distro is not supported.
	"""
	if distro == DEBIAN:
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
	if (
		NALA_SOURCES.exists()
		and not arguments.assume_yes
		and not ask(
			_("{file} already exists.\n" "Continue and overwrite it?").format(
				file=NALA_SOURCES
			)
		)
	):
		sys.exit(_("Abort."))
	# Make sure there aren't any shenanigans
	if arguments.fetches not in range(1, 11):
		sys.exit(_("Amount of fetches has to be 1-10..."))


def fetch() -> None:
	"""Fetch fast mirrors and write nala-sources.list."""
	fetch_checks()

	# If supplied a country it needs to be a list
	country_list = (arguments.country,) if arguments.country else None

	distro, release = detect_release()
	netselect, component = check_supported(distro, release, country_list)
	assert distro and release

	dprint(netselect)
	dprint(f"Distro: {distro}, Release: {release}, Component: {component}")

	aiorun(test_mirrors(netselect))
	netselect_scored.sort()

	dprint(netselect_scored)
	dprint(f"Size of original list: {len(netselect)}")
	dprint(f"Size of scored list: {len(netselect_scored)}")
	dprint(f"Writing from: {netselect_scored[:arguments.fetches]}")
	sources = parse_sources()
	write_sources(release, component, sources)
