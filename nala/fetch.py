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
import socket
import sys
from concurrent.futures import ThreadPoolExecutor

from apt_pkg import get_architectures
from aptsources.distro import get_distro
from httpx import HTTPError, get
from pythonping import ping
from rich.progress import TaskID

from nala.constants import ERRNO_PATTERN, ERROR_PREFIX, NALA_SOURCES
from nala.logger import dprint
from nala.options import arguments, parser
from nala.rich import Live, Table, fetch_progress
from nala.utils import ask, color

netselect_scored = []

DEBIAN = 'Debian'
UBUNTU = 'Ubuntu'
DOMAIN_PATTERN = re.compile(r'https?://([A-Za-z_0-9.-]+).*')
UBUNTU_COUNTRY = re.compile(r'<mirror:countrycode>(.*)</mirror:countrycode>')
UBUNTU_MIRROR = re.compile(r'<link>(.*)</link>')

def net_select(mirror: str, task: TaskID, live: Live, total: int, num: int) -> None:
	"""Take a URL, ping the domain and score the latency."""
	debugger = [f'Thread: {num}', f'Current Mirror: {mirror}']
	ping_progress(live, task, total, num)

	# Regex to get the domain
	regex = re.search(DOMAIN_PATTERN, mirror)
	if not regex:
		debugger.append('Regex Failed')
		dprint(debugger)
		return
	domain = regex.group(1)
	debugger.append(f'Pinged: {domain}')
	try:
		if not netping(domain, mirror, debugger):
			return
	except (socket.gaierror, RuntimeError) as err:
		ping_error(str(err), domain, mirror)

def netping(domain: str, mirror: str, debugger: list[str]) -> bool:
	"""Ping the domain and score it."""
	# We convert the float to integer in order to get rid of the decimal
	# From there we convert it to a string so we can prefix zeros for sorting
	res = str(int(ping(domain, count=4, match=True, timeout=1).rtt_avg_ms))
	debugger.append(f'Ping ms: {res}')
	if len(res) == 2:
		res = '0'+res
	elif len(res) == 1:
		res = '00'+res
	elif len(res) > 3:
		debugger.append('Mirror too slow')
		dprint(debugger)
		return False

	debugger.append(f'Appended: {res} {mirror}')
	dprint(debugger)
	netselect_scored.append(f'{res} {mirror}')
	return True

def ping_progress(live: Live, task: TaskID, total: int, num: int) -> None:
	"""Update fetch progress bar."""
	if not arguments.debug:
		table = Table.grid()
		table.add_row(f"{color('Mirror:', 'GREEN')} {num}/{total}")
		table.add_row(fetch_progress.get_renderable())

		fetch_progress.advance(task)
		live.update(table)

def ping_error(err: str, domain: str, mirror: str) -> None:
	"""Handle error on ping."""
	if arguments.verbose:
		# socket.gaierror: [Errno -2] Name or service not known
		if 'Errno' in err:
			print(
				f"{color(re.sub(ERRNO_PATTERN, '', err), 'YELLOW')}: {domain}",
				f"{color('URL:', 'YELLOW')} {mirror}\n"
			)
			return
		# 'Cannot resolve address "mirror.telepoint.bg", try verify your DNS or host file'
		if err.startswith('Cannot resolve'):
			print(f"{color('Warning:', 'YELLOW')} {str(err).split(',', maxsplit=1)[0]}")
			return
		print(err)

def ubuntu_mirror(country_list: tuple[str, ...] | None) -> tuple[str, ...]:
	"""Get and parse the Ubuntu mirror list."""
	print('Fetching Ubuntu mirrors...')
	ubuntu = fetch_mirrors("https://launchpad.net/ubuntu/+archivemirrors-rss", '<item>')
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
	return parse_mirror(UBUNTU, ubuntu, country_list)

def debian_mirror(country_list: tuple[str, ...] | None) -> tuple[str, ...]:
	"""Get and parse the Debian mirror list."""
	print('Fetching Debian mirrors...')
	debian = fetch_mirrors("https://mirror-master.debian.org/status/Mirrors.masterlist", '\n\n')
	arches = tuple(get_architectures())
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
	return parse_mirror(DEBIAN, debian, country_list, arches)

def fetch_mirrors(url: str, splitter: str) -> tuple[str, ...]:
	"""Attempt to fetch the url and split a list based on the splitter."""
	try:
		mirror_list = get(url, timeout=15).text.split(splitter)
	except HTTPError:
		sys.exit(ERROR_PREFIX+f'unable to connect to {url}')
	return tuple(mirror_list)

def parse_mirror(
		distro: str, master_mirror: tuple[str, ...],
		country_list: tuple[str, ...] | None,
		arches: tuple[str, ...] | tuple[()] = ()
	) -> tuple[str, ...]:
	"""Parse the mirror."""
	mirror_set = set()
	if arguments.verbose:
		print('Parsing mirror list...')
	# If no country is supplied then our list will be all countries
	countries = country_list or get_countries(master_mirror)
	for country, mirror in itertools.product(countries, master_mirror):
		if country not in mirror:
			continue
		if distro == DEBIAN:
			url = debian_parser(mirror, arches)
		elif distro == UBUNTU:
			url = ubuntu_parser(mirror)

		if url:
			mirror_set.add(url)
	return tuple(mirror_set)

def get_countries(master_mirror: tuple[str, ...]) -> tuple[str, ...]:
	"""Iterate the mirror list and return all valid countries."""
	country_list = set()
	# The way we split the information we get nice and pretty mirror selections
	for mirror in master_mirror:
		for line in mirror.splitlines():
			# Debian Countries
			if 'Country:' in line:
				# Country: SE Sweden
				country_list.add(line.split()[1])
			# Ubuntu Countries
			elif '<mirror:countrycode>' in line:
				# <mirror:countrycode>US</mirror:countrycode>
				result = re.search(UBUNTU_COUNTRY, line)
				if result:
					country_list.add(result.group(1))
	return tuple(country_list)

def debian_parser(mirror: str, arches: tuple[str, ...] | tuple[()]) -> str | None:
	"""Parse the Debian mirror."""
	url = 'http://'
	if 'Archive-http:' in mirror and all(arch in mirror for arch in arches):
		for line in mirror.splitlines():
			if line.startswith(('Archive-http:', 'Site:')):
				#['Site:', 'mirror.steadfastnet.com']
				#['Archive-http:', '/debian/']
				url += line.split()[1]
	if url == 'http://':
		return None
	return url

def ubuntu_parser(mirror: str) -> str | None:
	"""Parse the Ubuntu mirror."""
	# First section we get from Ubuntu is garbage. Let's ditch it and get to business
	if '<title>Ubuntu Archive Mirrors Status</title>' in mirror:
		return None

	for line in mirror.splitlines():
		if '<link>' in line:
			# <link>http://mirror.steadfastnet.com/ubuntu/</link>
			result = re.search(UBUNTU_MIRROR, line)
			if result:
				return result.group(1)
	return None

def detect_release() -> tuple[str, str]:
	"""Detect the distro and release."""
	if not arguments.debian and not arguments.ubuntu:
		lsb = get_distro()
		return lsb.id, lsb.codename

	if arguments.debian:
		return DEBIAN, arguments.debian
	return UBUNTU, arguments.ubuntu

def write_sources(release: str, component: str) -> None:
	"""Write mirrors to nala-sources.list."""
	with open(NALA_SOURCES, 'w', encoding="utf-8") as file:
		print(f"{color('Writing:', 'GREEN')} {NALA_SOURCES}\n")
		print('# Sources file built for nala\n', file=file)
		arguments.fetches -= 1
		for num, line in enumerate(netselect_scored):
			# This splits off the score '030 http://mirror.steadfast.net/debian/'
			line = line[line.index('h'):]
			source = f'{line} {release} {component}'
			print(f'deb {source}')
			print(f'deb-src {source}\n')
			print(f'deb {source}', file=file)
			print(f'deb-src {source}\n', file=file)
			if num == arguments.fetches:
				break

def test_mirrors(netselect: tuple[str, ...]) -> None:
	"""Test mirrors."""
	print('Testing mirrors...')
	with Live(transient=True) as live:
		with ThreadPoolExecutor(max_workers=32) as pool:
			total = len(netselect)
			task = fetch_progress.add_task('', total=total)

			for num, mirror in enumerate(netselect):
				pool.submit(net_select, mirror, task, live, total, num)

def check_supported(distro:str, release:str,
	country_list: tuple[str, ...] | None) -> tuple[tuple[str, ...], str]:
	"""Check if the distro is supported or not.

	If the distro is supported return mirror list and component.

	Error if the distro is not supported.
	"""
	if distro == DEBIAN:
		component = 'main' if arguments.foss else 'main contrib non-free'
		return debian_mirror(country_list), component
	if distro == UBUNTU:
		# It's ubuntu, you probably don't care about foss
		return ubuntu_mirror(country_list), 'main restricted universe multiverse'
	parser.parse_args(['fetch', '--help'])
	print(
		ERROR_PREFIX+f"{distro} {release} is unsupported.\n"
		"You can specify Ubuntu or Debian manually."
	)
	sys.exit(ERROR_PREFIX+f"{distro} {release} is unsupported.")

def fetch_checks() -> None:
	"""Perform checks and error if we shouldn't continue."""
	if (NALA_SOURCES.exists() and not arguments.assume_yes and
	    not ask(f'{NALA_SOURCES.name} already exists.\ncontinue and overwrite it')
	    ):
		sys.exit('Abort')
	# Make sure there aren't any shenanigans
	if arguments.fetches not in range(1,11):
		sys.exit('Amount of fetches has to be 1-10...')

def fetch() -> None:
	"""Fetch fast mirrors and write nala-sources.list."""
	fetch_checks()

	# If supplied a country it needs to be a list
	country_list = (arguments.country,) if arguments.country else None

	distro, release = detect_release()
	netselect, component = check_supported(distro, release, country_list)

	dprint(netselect)
	dprint(f'Distro: {distro}, Release: {release}, Component: {component}')

	test_mirrors(netselect)
	netselect_scored.sort()

	dprint(netselect_scored)
	dprint(f'Size of original list: {len(netselect)}')
	dprint(f'Size of scored list: {len(netselect_scored)}')
	dprint(f'Writing from: {netselect_scored[:arguments.fetches]}')
	write_sources(release, component)
