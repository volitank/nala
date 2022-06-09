#                 __
#    ____ _____  |  | _____
#   /    \\__  \ |  | \__  \
#  |   |  \/ __ \|  |__/ __ \_
#  |___|  (____  /____(____  /
#       \/     \/          \/
#
# Copyright (C) 2010 - 2021 Tatsuhiro Tsujikawa
# Copyright (C) 2021, 2022 Blake Lee
#
# This file is part of nala
# nala is based upon apt-metalink https://github.com/tatsuhiro-t/apt-metalink
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
"""Module for package downloader."""
from __future__ import annotations

import asyncio
import contextlib
import hashlib
import re
import sys
from asyncio import AbstractEventLoop, CancelledError, gather, run, sleep
from collections import Counter
from errno import ENOENT
from functools import partial
from pathlib import Path
from signal import Signals  # pylint: disable=no-name-in-module #Codacy
from signal import SIGINT, SIGTERM
from typing import Union, cast

from anyio import open_file
from apt.package import Package, Version
from apt_pkg import Configuration, config
from httpx import (
	URL,
	AsyncClient,
	ConnectError,
	ConnectTimeout,
	HTTPError,
	HTTPStatusError,
	Proxy,
	RemoteProtocolError,
	RequestError,
	get,
)

from nala import _, color
from nala.constants import (
	ARCHIVE_DIR,
	ERRNO_PATTERN,
	ERROR_PREFIX,
	NOTICE_PREFIX,
	PARTIAL_DIR,
	WARNING_PREFIX,
)
from nala.error import ExitCode, FileDownloadError
from nala.options import arguments
from nala.rich import (
	ELLIPSIS,
	Live,
	Panel,
	Pretty,
	Table,
	from_ansi,
	pkg_download_progress,
)
from nala.utils import (
	dprint,
	eprint,
	get_pkg_name,
	pkg_candidate,
	term,
	unit_str,
	vprint,
)

MIRROR_PATTERN = re.compile(r"mirror://(.*?/.*?)/")
URL_PATTERN = re.compile(r"(https?://.*?/.*?)/")

STARTING_DOWNLOADS = color(_("Starting Downloads") + ELLIPSIS, "BLUE")

STARTING_DOWNLOAD = color(_("Starting Download:"), "BLUE")
# NOTE: "Starting Download: http://deb.debian.org/debian 3MB"
# NOTE: "Starting Download:" will be colored blue
STARTING_DOWNLOAD_STATUS = _("{starting_download} {url} {size}")

DOWNLOAD_COMPLETE = color(_("Download Complete:"), "GREEN")
# NOTE: "Download Complete: http://deb.debian.org/debian"
# NOTE: "Download Complete:" will be colored green
DOWNLOAD_COMPLETE_STATUS = _("{download_complete} {url}")

TOTAL_PACKAGES = color(_("Total Packages:"), "GREEN")
# NOTE: "Total Packages: 30/100"
# NOTE: "Starting Download:" will be colored green
TOTAL_PACKAGES_STATUS = _("{total_packages} {current}/{total}")

LAST_COMPLETED = color(_("Last Completed:"), "GREEN")
LAST_COMPLETED_STATUS = _("{last_completed} {package}")


REMOVING_FILE = _("{notice} Nala has removed {filename} but will try another mirror")
FAILED_MOVE = _(
	"{error} Failed to move archive file, {str_err}: '{file1}' -> '{file2}'"
)
FILE_NO_EXIST = _("{error} {filename} does not exist!")
HASH_MISMATCH = _(
	"{error} Hash Sum does not match: {filename}\n"
	"  Expected Hash: {expected}\n"
	"  Received Hash: {received}"
)
SIZE_WRONG = _(
	"{error} File has unexpected size: {filename}\n"
	"  Expected Size: {expected}\n"
	"  Received Size: {received}"
)

## Debugging messages, No translation
HASH_STATUS = (
	"Hash Status = [\n    File: {filepath},\n"
	"    Candidate Hash: {hash_type} {expected},\n"
	"    Local Hash: {received},\n"
	"    Hash Success: {result},\n]"
)
HTTPX_STATUS_ERROR = (
	"Hash Status = [\n    File: {filepath},\n"
	"    Candidate Hash: {hash_type} {expected},\n"
	"    Local Hash: {received},\n"
	"    Hash Success: {result},\n]"
)
PEER_CLOSED = "peer closed connection without sending complete message body"
DownloadErrorTypes = Union[
	HTTPError, HTTPStatusError, RequestError, OSError, ConnectError, FileDownloadError
]


class PkgDownloader:  # pylint: disable=too-many-instance-attributes
	"""Manage Package Downloads."""

	def __init__(self, pkgs: list[Package]) -> None:
		"""Manage Package Downloads."""
		dprint("Downloader Initializing")
		self.total_pkgs: int = len(pkgs)
		self.total_data: int = 0
		self.count: int = 0
		self.live: Live
		self.mirrors: dict[str, list[str]] = {}
		self.last_completed: str = ""
		self.untrusted: list[str] = []
		self.proxy: dict[URL | str, URL | str | Proxy | None] = {}
		self.failed: list[str] = []
		self.current: Counter[str] = Counter()
		self.fatal: bool = False
		self.exit: int | bool = False

		self.pkg_urls: dict[Version, list[str]] = {
			pkg.candidate: self.filter_uris(pkg.candidate)
			for pkg in pkgs
			if pkg.candidate
		}
		self.task = pkg_download_progress.add_task(
			"", total=sum(candidate.size for candidate in self.pkg_urls)
		)

		if self.untrusted:
			untrusted_error(self.untrusted)

		self._set_proxy()
		dprint("Initialization Complete")

	async def start_download(self) -> bool:
		"""Start async downloads."""
		if not self.pkg_urls:
			return True
		with Live(get_renderable=self._gen_table, refresh_per_second=10) as self.live:
			async with AsyncClient(
				timeout=20, proxies=self.proxy, follow_redirects=True
			) as client:
				loop = asyncio.get_running_loop()
				tasks = (
					loop.create_task(self._init_download(client, candidate, urls))
					for candidate, urls in self.pkg_urls.items()
				)
				# Setup handlers for Interrupts
				for signal_enum in (SIGINT, SIGTERM):
					exit_func = partial(self.interrupt, signal_enum, loop)
					loop.add_signal_handler(signal_enum, exit_func)

				return all(await gather(*tasks))

	async def _download(
		self, client: AsyncClient, candidate: Version, url: str
	) -> None:
		"""Download and write package."""
		dest = PARTIAL_DIR / get_pkg_name(candidate)
		vprint(
			STARTING_DOWNLOAD_STATUS.format(
				starting_download=STARTING_DOWNLOAD,
				url=url,
				size=unit_str(candidate.size, 1),
			)
		)
		second_attempt = False
		while True:
			total_data = 0
			hash_type, expected = get_hash(candidate)
			hash_fun = hashlib.new(hash_type)
			try:
				async with client.stream("GET", url) as response:
					response.raise_for_status()
					async with await open_file(dest, mode="wb") as file:
						async for data in response.aiter_bytes():
							if data:
								await file.write(data)
								hash_fun.update(data)
								total_data += len(data)
								await self._update_progress(len(data))
			# Sometimes mirrors play a little dirty and close the connection
			except RemoteProtocolError as error:
				await self._update_progress(total_data, failed=True)
				if second_attempt:
					raise error from error
				second_attempt = True
				dprint(f"Mirror Failed: {url} {error}, will try again.")
				continue

			dprint(
				HASH_STATUS.format(
					filepath=dest,
					hash_type=hash_type.upper(),
					expected=expected,
					received=(received := hash_fun.hexdigest()),
					result=received == expected,
				)
			)
			if expected != received:
				dest.unlink()
				self.fatal = True
				await self._update_progress(total_data, failed=True)
				raise FileDownloadError(
					errno=FileDownloadError.ERRHASH,
					filename=dest.name,
					expected=f"{hash_type.upper()}: {expected}",
					received=f"{hash_type.upper()}: {received}",
				)
			break

	async def _init_download(
		self,
		client: AsyncClient,
		candidate: Version,
		urls: list[str],
	) -> None:
		"""Download pkgs."""
		while urls:
			for num, url in enumerate(urls):
				if not (domain := await self._check_count(url)):
					continue
				try:
					await self._download(client, candidate, url)

					await process_downloads(candidate)
					check_pkg(ARCHIVE_DIR, candidate, is_download=True)
					vprint(
						DOWNLOAD_COMPLETE_STATUS.format(
							download_complete=DOWNLOAD_COMPLETE, url=url
						)
					)

					self.count += 1
					self.current[domain] -= 1
					self.last_completed = Path(candidate.filename).name
					self.live.update(self._gen_table())
					break

				except (HTTPError, OSError, FileDownloadError) as error:
					urls.pop(num)
					self.current[domain] -= 1
					self.download_error(error, num, urls, candidate)
					continue
			else:
				continue
			break

	def interrupt(self, signal_enum: Signals, loop: AbstractEventLoop) -> None:
		"""Shutdown the loop."""
		self.exit = 128 + signal_enum.real
		if self.exit == ExitCode.SIGINT:
			term.write(term.CURSER_UP + term.CLEAR_LINE)

		self.live.stop()
		for task in asyncio.all_tasks(loop):
			task.cancel()
		eprint(_("Exiting due to {signal}").format(signal=signal_enum.name))

	def _set_proxy(self) -> None:
		"""Set proxy configuration."""
		for proto in ("http", "https"):
			try:
				proxy_config = cast(
					Configuration, config.subtree(f"Acquire::{proto}::Proxy")  # type: ignore[attr-defined]
				)
			except KeyError:
				continue

			for key in proxy_config.keys():
				value: str | None = proxy_config.find(key)
				if not value:
					continue
				if "socks5h" in value or "socks5" in value:
					try:
						import socksio  # pylint: disable=import-outside-toplevel, unused-import

						value = value.replace("socks5h", "socks5")
					except ImportError:
						eprint(
							_(
								"{error} Using SOCKS proxy, but the 'socksio' package is not installed."
							).format(error=ERROR_PREFIX)
						)
						sys.exit(
							_(
								"{error} Install using 'nala install python3-socksio'"
							).format(error=ERROR_PREFIX)
						)
				if value == "DIRECT":
					value = None
					# self.proxy[f"{proto}://{key}"] = None
					# continue
				self.proxy[f"{proto}://{key}"] = value

		if http_proxy := config.find("Acquire::http::Proxy"):
			self.proxy["http://"] = http_proxy
		if https_proxy := config.find("Acquire::https::Proxy"):
			self.proxy["https://"] = https_proxy

	def set_mirrors_txt(self, domain: str) -> None:
		"""Check if user has mirrors:// and handle accordingly."""
		if domain not in self.mirrors:
			url = f"http://{domain}"
			try:
				self.mirrors[domain] = get(url, follow_redirects=True).text.splitlines()
			except HTTPError:
				sys.exit(
					_("{error} unable to connect to {url}").format(
						error=ERROR_PREFIX, url=url
					)
				)

	def filter_uris(self, candidate: Version) -> list[str]:
		"""Filter uris into usable urls."""
		urls: list[str] = []
		for uri in candidate.uris:
			if not check_trusted(uri, candidate):
				self.untrusted.append(color(candidate.package.name, "RED"))
			# Regex to check if we're using mirror://
			if regex := MIRROR_PATTERN.search(uri):
				self.set_mirrors_txt(domain := regex.group(1))
				urls.extend(
					link + candidate.filename
					for link in self.mirrors[domain]
					if not link.startswith("#")
				)
				continue
			urls.append(uri)
		return urls

	async def _check_count(self, url: str) -> str:
		"""Check the url count and return if Nala should continue."""
		domain = ""
		if regex := URL_PATTERN.search(url):
			domain = regex.group(1)
			if self.current[domain] > 2:
				await sleep(0.1)
				return ""
			self.current[domain] += 1
		return domain

	def _gen_table(self) -> Panel:
		"""Generate Rich Table."""
		table = Table.grid()
		if arguments.debug:
			table.add_row(Pretty(self.current))
		table.add_row(
			from_ansi(
				TOTAL_PACKAGES_STATUS.format(
					total_packages=TOTAL_PACKAGES,
					current=self.count,
					total=self.total_pkgs,
				)
			)
		)

		if not self.last_completed:
			table.add_row(from_ansi(STARTING_DOWNLOADS))
		else:
			table.add_row(
				from_ansi(
					LAST_COMPLETED_STATUS.format(
						last_completed=LAST_COMPLETED, package=self.last_completed
					)
				)
			)

		table.add_row(pkg_download_progress.get_renderable())
		return Panel(
			table,
			title="[bold default]" + _("Downloading") + ELLIPSIS,
			title_align="left",
			border_style="bold green",
		)

	def download_error(
		self,
		error: DownloadErrorTypes,
		num: int,
		urls: list[str],
		candidate: Version,
	) -> None:
		"""Handle download errors."""
		print_error(error)

		if not (next_url := more_urls(urls, num, self.failed, candidate)):
			# Status error are fatal as apt_pkg is likely to fail with these as well
			if isinstance(error, HTTPStatusError):
				self.fatal = True
			return

		vprint(_("Trying the next url: {url}").format(url=next_url))

	async def _update_progress(self, len_data: int, failed: bool = False) -> None:
		"""Update download progress."""
		if failed:
			self.total_data -= len_data
			pkg_download_progress.reset(self.task, completed=self.total_data)
			return
		self.total_data += len_data
		pkg_download_progress.advance(self.task, advance=len_data)
		self.live.update(self._gen_table())


def untrusted_error(untrusted: list[str]) -> None:
	"""Print the untrusted warnings and exit if we're not allowed."""
	eprint(
		_("{warning} The following packages cannot be authenticated!").format(
			warning=WARNING_PREFIX
		)
	)
	eprint(f"  {', '.join(untrusted)}")
	if not config.find_b("APT::Get::AllowUnauthenticated", False):
		sys.exit(
			_("{error} Some packages were unable to be authenticated").format(
				error=ERROR_PREFIX
			)
		)
	eprint(
		_(
			"{notice} Configuration is set to allow installation of unauthenticated packages."
		).format(notice=NOTICE_PREFIX)
	)


def print_error(error: DownloadErrorTypes) -> None:
	"""Print the download error to console."""
	if isinstance(error, ConnectTimeout):
		eprint(
			_("{error} {url} timed out").format(
				error=ERROR_PREFIX, url=error.request.url
			)
		)
		return
	if isinstance(error, ConnectError):
		# ConnectError: [Errno -2] Name or service not known
		errno_replace = re.sub(ERRNO_PATTERN, "", f"{error}").strip() + ":"
		eprint(f"{ERROR_PREFIX} {errno_replace} {error.request.url}")
		return
	if isinstance(error, FileDownloadError):
		file_error(error)
		return
	if isinstance(error, OSError) and error.errno != ENOENT:
		eprint(
			FAILED_MOVE.format(
				error=ERROR_PREFIX,
				str_err=error.strerror,
				file1=error.filename,
				file2=error.filename2,
			)
		)
		return
	msg = f"{error}" or type(error).__name__
	msg = msg.replace("\n", "\n  ")
	eprint(f"{ERROR_PREFIX} {msg}")


def file_error(error: FileDownloadError) -> None:
	"""Print the error from our FileDownloadError exception."""
	filename = color(error.filename, "YELLOW")
	if error.errno == FileDownloadError.ENOENT:
		eprint(FILE_NO_EXIST.format(error=ERROR_PREFIX, filename=filename))
		return
	if error.errno == FileDownloadError.ERRHASH:
		eprint(
			HASH_MISMATCH.format(
				error=ERROR_PREFIX,
				filename=filename,
				expected=error.expected,
				received=error.received,
			)
		)

	if error.errno == FileDownloadError.ERRSIZE:
		eprint(
			SIZE_WRONG.format(
				error=ERROR_PREFIX,
				filename=filename,
				expected=error.expected,
				received=error.received,
			)
		)

	eprint(REMOVING_FILE.format(notice=NOTICE_PREFIX, filename=filename))


def more_urls(urls: list[str], num: int, failed: list[str], candidate: Version) -> str:
	"""Check if there is another url to try. Return False if not."""
	try:
		return urls[num + 1]
	except IndexError:
		filename = Path(candidate.filename).name
		eprint(
			_("{error} No more mirrors available for {filename}").format(
				error=ERROR_PREFIX, filename=color(filename, "YELLOW")
			)
		)
		failed.append(filename)
		return ""


async def process_downloads(candidate: Version) -> bool:
	"""Process the downloaded packages."""
	filename = get_pkg_name(candidate)
	destination = ARCHIVE_DIR / filename
	source = PARTIAL_DIR / filename
	try:
		dprint(f"Moving {source} -> {destination}")
		source.rename(destination)
	except OSError as error:
		if error.errno != ENOENT:
			eprint(
				FAILED_MOVE.format(
					error=ERRNO_PATTERN,
					str_err=error.strerror,
					file1=error.filename,
					file2=error.filename2,
				)
			)
		return False
	return True


def check_trusted(uri: str, candidate: Version) -> bool:
	"""Check if the candidate is trusted."""
	for (packagefile, _unused) in candidate._cand.file_list:
		if packagefile.site in uri and packagefile.archive != "now":
			indexfile = candidate.package._pcache._list.find_index(packagefile)
			return bool(indexfile and indexfile.is_trusted)
	return False


def check_pkg(
	directory: Path, candidate: Package | Version, is_download: bool = False
) -> bool:
	"""Check if file exists, is correct, and run check hash."""
	if is_download:
		dprint("Post Download Package Check")
	else:
		dprint("Pre Download Package Check")
	if isinstance(candidate, Package):
		candidate = pkg_candidate(candidate)
	path = directory / get_pkg_name(candidate)
	if not path.exists():
		dprint(f"File Doesn't exist: {path.name}")
		if is_download:
			raise FileDownloadError(
				errno=FileDownloadError.ENOENT,
				filename=path.name,
			)
		return False
	if (size := path.stat().st_size) != candidate.size:
		dprint(f"File {path.name} has an unexpected size {size} != {candidate.size}")
		path.unlink()
		if is_download:
			raise FileDownloadError(
				errno=FileDownloadError.ERRSIZE,
				filename=path.name,
				expected=f"{candidate.size}",
				received=f"{size}",
			)
		return False

	# If we're downloading we checked the hash on the fly.
	# We can skip doing it again
	if is_download:
		return True

	hash_type, expected = get_hash(candidate)
	try:
		if not check_hash(path, hash_type, expected):
			dprint(f"Hash Checking has failed. Removing: {path.name}")
			path.unlink()
			return False
		dprint(f"Package doesn't require download: {path.name}")
		return True
	except OSError as err:
		eprint(_("Failed to check hash"), err)
		return False


def check_hash(path: Path, hash_type: str, expected: str) -> bool:
	"""Check hash value."""
	hash_fun = hashlib.new(hash_type)
	with path.open("rb") as file:
		while True:
			data = file.read(4096)
			if not data:
				break
			hash_fun.update(data)
	received = hash_fun.hexdigest()
	dprint(
		HASH_STATUS.format(
			filepath=path,
			hash_type=hash_type.upper(),
			expected=expected,
			received=received,
			result=received == expected,
		)
	)
	return received == expected


def get_hash(version: Version) -> tuple[str, str]:
	"""Get the correct hash value."""
	hash_list = version._records.hashes
	hashes = ("SHA512", "SHA256")

	# From Debian's requirements we are not to use these for security checking.
	# https://wiki.debian.org/DebianRepository/Format#MD5Sum.2C_SHA1.2C_SHA256
	# Clients may not use the MD5Sum and SHA1 fields for security purposes,
	# and must require a SHA256 or a SHA512 field.
	# hashes = ('SHA512', 'SHA256', 'SHA1', 'MD5')

	for _type in hashes:
		with contextlib.suppress(KeyError):
			return _type.lower(), hash_list.find(_type).hashvalue

	filename = Path(version.filename).name or version.package.name
	eprint(
		_("{error} {filename} can't be checked for integrity.").format(
			error=ERROR_PREFIX, filename=color(filename, "YELLOW")
		)
	)
	sys.exit(_("There are no hashes available for this package."))


def download(pkgs: list[Package]) -> None:
	"""Run downloads and check for failures.

	Does not return if in Download Only mode.
	"""
	downloader = PkgDownloader(
		# Start the larger files first, as they take the longest
		# Ignore mypy here because the pkgs definitely have a candidate if they made it here
		sorted(pkgs, key=lambda pkg: pkg.candidate.size, reverse=True)  # type: ignore[union-attr]
	)
	try:
		run(downloader.start_download())
	except (CancelledError, RuntimeError) as error:
		if downloader.exit:
			sys.exit(downloader.exit)
		raise error from error
	except ValueError as error:
		sys.exit(f"{ERROR_PREFIX} {error}")

	if arguments.download_only and not downloader.failed:
		print(_("Download complete and in download only mode."))
		sys.exit(0)

	if downloader.failed:
		eprint(
			_("{error} Download failure. The following downloads failed:").format(
				error=ERROR_PREFIX
			)
		)
		eprint(f"  {', '.join(color(pkg, 'YELLOW') for pkg in downloader.failed)}")

		if downloader.fatal:
			sys.exit(1)
		if arguments.download_only:
			sys.exit(_("In download only mode. Not falling back to apt_pkg."))
		eprint(
			_(
				"{warning} Falling back to apt_pkg. The following downloads failed:"
			).format(warning=WARNING_PREFIX)
		)
