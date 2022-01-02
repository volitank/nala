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

import re
import select
import socket
import struct
import threading
import time
from secrets import SystemRandom
from sys import exit, stderr

import requests
from aptsources.distro import get_distro
from click import style

from nala.options import arg_parse
from nala.rich_custom import fetch_progress, rich_grid, rich_live
from nala.utils import GREEN, NALA_SOURCES, RED, YELLOW, ask, dprint, shell

netselect_scored = []

parser = arg_parse()
arguments = parser.parse_args()
verbose = arguments.verbose

def chk(data):
	x = sum(x << 8 if i % 2 else x for i, x in enumerate(data)) & 0xFFFFFFFF
	x = (x >> 16) + (x & 0xFFFF)
	x = (x >> 16) + (x & 0xFFFF)
	return struct.pack('<H', ~x & 0xFFFF)

def ping(addr, timeout=2, number=1, data=b''):
	with socket.socket(socket.AF_INET, socket.SOCK_RAW, socket.IPPROTO_ICMP) as conn:
		payload = struct.pack('!HH', SystemRandom().randrange(0, 65536), number) + data

		conn.connect((addr, 80))
		conn.sendall(b'\x08\0' + chk(b'\x08\0\0\0' + payload) + payload)
		start = time.time()

		while select.select([conn], [], [], max(0, start + timeout - time.time()))[0]:
			data = conn.recv(65536)
			if len(data) < 20 or len(data) < struct.unpack_from('!xxH', data)[0]:
				continue
			if data[20:] == b'\0\0' + chk(b'\0\0\0\0' + payload) + payload:
				return time.time() - start

def net_select(host):
	try:
		dprint(host)
		# Regex to get the domain
		regex = re.search('https?://([A-Za-z_0-9.-]+).*', host)
		if not regex:
			return
		domain = regex.group(1)
		dprint(domain)
		res = ping(domain)
		if res:
			res = str(int(res*1000))
			if len(res) == 2:
				res = '0'+res
			if len(res) == 1:
				res == '00'+res
			netselect_scored.append(f'{res} {host}')
	except (socket.gaierror, OSError) as e:
		if verbose:
			e = str(e)
			regex = re.search('\\[.*\\]', e)
			if regex:
				e = style(e.replace(regex.group(0), '').strip(), **YELLOW)
			print(f'{e}: {domain}')
			print(f'{style("URL:", **YELLOW)} {host}\n')

def parse_ubuntu(country_list: list=None):
	print('Fetching Ubuntu mirrors...')

	try:
		ubuntu = requests.get("https://launchpad.net/ubuntu/+archivemirrors-rss").text.split('<item>')
	except requests.ConnectionError:
		err = style("Error:", **RED)
		stderr.write(f'{err} unable to connect to http://mirrors.ubuntu.com/mirrors.txt\n')
		exit(1)

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
		err = style("Error:", **RED)
		stderr.write(f'{err} unable to connect to http://mirrors.ubuntu.com/mirrors.txt\n')
		exit(1)

	arches = shell.dpkg.__print_architecture().stdout.strip().split()
	foreign_arch = shell.dpkg.__print_foreign_architectures().stdout.strip().split()

	if foreign_arch:
		arches += foreign_arch

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
		err = style('Error:', **RED)
		print(f'{err} Unable to detect release. Specify manually')
		parser.parse_args(['fetch', '--help'])
		exit(1)

	if distro and release:
		return distro, release

def fetch(	fetches: int, foss: bool = False,
			debian=None, ubuntu=None, country=None,
			assume_yes=False):
	"""Fetches fast mirrors and write to nala-sources.list"""
	if (NALA_SOURCES.exists() and not assume_yes and
	    not ask(f'{NALA_SOURCES.name} already exists.\ncontinue and overwrite it')
	    ):
		print('Abort')
		exit()

	# Make sure there aren't any shenanigans
	if fetches not in range(1,11):
		print('Amount of fetches has to be 1-10...')
		exit(1)

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

	thread_handler = []
	num = -1
	for _ in netselect:
		num += 1
		thread = threading.Thread(name='Net Select', target=net_select, args=[netselect[num]])
		thread_handler.append(thread)
		thread.start()

	task = fetch_progress.add_task(
		description='',
		total=len(thread_handler)
		)

	print('Testing URLs...')
	# wait for all our threads to stop
	with rich_live(transient=True) as live:
		for num, thread in enumerate(thread_handler):

			table = rich_grid()
			table.add_row(f'{style("URL:", **GREEN)} {num}/{len(thread_handler)}')
			table.add_row(fetch_progress)

			thread.join()
			fetch_progress.advance(task, advance=num)
			live.update(table)

	netselect_scored.sort()

	dprint(netselect_scored)

	num = 0
	with open(NALA_SOURCES, 'w') as file:
		print(f"{style('Writing:', **GREEN)} {NALA_SOURCES}\n")
		print('# Sources file built for nala\n', file=file)
		for line in netselect_scored:
			num += 1
			line = line[line.index('h'):]
			print(f'deb {line} {release} {component}')
			print(f'deb-src {line} {release} {component}\n')
			print(f'deb {line} {release} {component}', file=file)
			print(f'deb-src {line} {release} {component}\n', file=file)
			if num == fetches:
				break
