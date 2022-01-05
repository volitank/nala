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

from __future__ import annotations

import re
import socket
import sys
from concurrent.futures import ThreadPoolExecutor

import requests
from aptsources.distro import get_distro
from pythonping import ping

from nala.logger import dprint
from nala.options import arguments, parser
from nala.rich_custom import fetch_progress, rich_grid, rich_live
from nala.utils import (color, YELLOW, GREEN,
				ERROR_PREFIX, NALA_SOURCES, ask, shell)

netselect_scored = []
verbose = arguments.verbose

def net_select(mirror, task, live, total, num) -> None:
	"""Takes a URL and pings the domain and scores the latency."""
	debugger = [f'Thread: {num}', f'Current Mirror: {mirror}']

	if not arguments.debug:
		table = rich_grid()
		table.add_row(f'{color("Mirror:", GREEN)} {num}/{total}')
		table.add_row(fetch_progress)

		fetch_progress.advance(task)
		live.update(table)
	try:
		# Regex to get the domain
		regex = re.search('https?://([A-Za-z_0-9.-]+).*', mirror)
		if not regex:
			debugger.append('Regex Failed')
			dprint(debugger)
			return
		domain = regex.group(1)
		debugger.append(f'Pinged: {domain}')
		res = str(int(ping(domain, count=4, match=True, timeout=1).rtt_avg_ms))

		debugger.append(f'Ping ms: {res}')
		if len(res) == 2:
			res = '0'+res
		elif len(res) == 1:
			res == '00'+res
		elif len(res) > 3:
			debugger.append('Mirror too slow')
			dprint(debugger)
			return

		score = f'{res} {mirror}'
		debugger.append(f'Appended: {score}')
		dprint(debugger)
		netselect_scored.append(score)

	except (socket.gaierror, OSError) as e:
		if verbose:
			e = str(e)
			regex = re.search('\\[.*\\]', e)
			if regex:
				e = color(e.replace(regex.group(0), '').strip(), YELLOW)
			print(f'{e}: {domain}')
			print(f'{color("URL:", YELLOW)} {mirror}\n')

def parse_ubuntu(country_list: list=None):
	print('Fetching Ubuntu mirrors...')

	try:
		ubuntu = requests.get("https://launchpad.net/ubuntu/+archivemirrors-rss").text.split('<item>')
	except requests.ConnectionError:
		sys.exit(ERROR_PREFIX+'unable to connect to http://mirrors.ubuntu.com/mirrors.txt')

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

	# No country has been supplied so we should get all the countries possible
	if country_list is None:
		country_list = set()
		# The way we split the information we get nice and pretty mirror selections
		for mirror in ubuntu:
			for line in mirror.splitlines():
				if '<mirror:countrycode>' in line:
					# <mirror:countrycode>US</mirror:countrycode>
					result = re.search('<mirror:countrycode>(.*)</mirror:countrycode>', line)
					country_list.add(result.group(1))

	if verbose:
		print('Parsing mirror list...')
	mirror_set = set()
	# If no country is supplied then our list will be all countries
	for country in country_list:
		for mirror in ubuntu:
			# First section we get from here is garbage. Let's ditch it and get to business
			if '<title>Ubuntu Archive Mirrors Status</title>' in mirror:
				continue
			# This is where the real magic is happening.
			if country in mirror:
				for line in mirror.splitlines():
					if '<link>' in line:
						# <link>http://mirror.steadfastnet.com/ubuntu/</link>
						result = re.search('<link>(.*)</link>', line)
						mirror_set.add(result.group(1))
	return list(mirror_set)

def parse_debian(country_list: list=None):
	print('Fetching Debian mirrors...')

	try:
		debian = requests.get("https://mirror-master.debian.org/status/Mirrors.masterlist").text.split('\n\n')
	except requests.ConnectionError:
		sys.exit(ERROR_PREFIX+'unable to connect to http://mirrors.ubuntu.com/mirrors.txt')

	arches = get_arch()

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

	# No country has been supplied so we should get all the countries possible
	if country_list is None:
		country_list = set()
		# The way we split the information we get nice and pretty mirror selections
		for mirror in debian:
			for line in mirror.splitlines():
				if 'Country:' in line:
					# Country: SE Sweden
					country_list.add(line.split()[1])
	if verbose:
		print('Parsing mirror list...')
	mirror_set = set()
	# If no country is supplied then our list will be all countries
	for country in country_list:
		for mirror in debian:
			if (country in mirror and 'Archive-http:' in mirror and all(arch in mirror for arch in arches)):
				url = 'http://'
				for line in mirror.splitlines():
					if 'Site:' in line:
						url += line.split()[1]
					if 'Archive-http:' in line:
						url += line.split()[1]
				mirror_set.add(url)
	return list(mirror_set)

def detect_release():

	lsb = get_distro()
	try:
		distro = lsb.id
		release = lsb.codename
	except:
		print(ERROR_PREFIX+'Unable to detect release. Specify manually')
		parser.parse_args(['fetch', '--help'])
		sys.exit(1)

	if distro and release:
		return distro, release

def get_arch() -> list[str]:
	"""Query dpkg for supported architectures."""
	arches = shell.dpkg.__print_architecture().stdout.strip().split()
	foreign_arch = shell.dpkg.__print_foreign_architectures().stdout.strip().split()

	if foreign_arch:
		arches += foreign_arch
	return arches

def fetch(	fetches: int, foss: bool = False,
			debian=None, ubuntu=None, country=None,
			assume_yes=False):
	"""Fetches fast mirrors and write to nala-sources.list"""
	if (NALA_SOURCES.exists() and not assume_yes and
	    not ask(f'{NALA_SOURCES.name} already exists.\ncontinue and overwrite it')
	    ):
		sys.exit('Abort')

	# Make sure there aren't any shenanigans
	if fetches not in range(1,11):
		sys.exit('Amount of fetches has to be 1-10...')

	# If supplied a country it needs to be a list
	if country:
		country = [country]

	if not debian and not ubuntu:
		distro, release = detect_release()
	elif debian:
		distro = 'Debian'
		release = debian
	else:
		distro = 'Ubuntu'
		release = ubuntu

	if distro == 'Debian':
		netselect = parse_debian(country)
		component = 'main' if foss else 'main contrib non-free'
	else:
		netselect = parse_ubuntu(country)
		# It's ubuntu, you probably don't care about foss
		component = 'main restricted universe multiverse'

	dprint(netselect)
	dprint(f'Distro: {distro}, Release: {release}, Component: {component}')

	print('Testing mirrors...')
	with rich_live(transient=True) as live:
		with ThreadPoolExecutor(max_workers=32) as pool:
			total = len(netselect)
			task = fetch_progress.add_task('', total=total)

			for num, mirror in enumerate(netselect):
				pool.submit(net_select, mirror, task, live, total, num)

	netselect_scored.sort()

	dprint(netselect_scored)
	dprint(f'Size of original list: {len(netselect)}')
	dprint(f'Size of scored list: {len(netselect_scored)}')
	dprint(f'Writing from: {netselect_scored[:fetches]}')

	with open(NALA_SOURCES, 'w') as file:
		print(f"{color('Writing:', GREEN)} {NALA_SOURCES}\n")
		print('# Sources file built for nala\n', file=file)
		fetches -= 1
		for num, line in enumerate(netselect_scored):
			# This splits off the score '030 http://mirror.steadfast.net/debian/'
			line = line[line.index('h'):]
			print(f'deb {line} {release} {component}')
			print(f'deb-src {line} {release} {component}\n')
			print(f'deb {line} {release} {component}', file=file)
			print(f'deb-src {line} {release} {component}\n', file=file)
			if num == fetches:
				break
