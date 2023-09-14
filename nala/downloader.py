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
import shutil
import sys
from asyncio import AbstractEventLoop, CancelledError, gather, run, sleep
from collections import Counter
from dataclasses import dataclass
from errno import ENOENT
from functools import partial
from pathlib import Path
from signal import Signals  # pylint: disable=no-name-in-module #Codacy
from signal import SIGINT, SIGTERM
from typing import Generator, Iterable, List, Sequence, Union, cast

from anyio import open_file
from apt.package import Package, Version
from apt_pkg import Configuration, config
from httpx import (
	URL as HttpxUrl,
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

from nala import _, __version__, color
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
from nala.utils import dprint, eprint, get_pkg_name, term, unit_str, vprint

MIRROR_PATTERN = re.compile(r"mirror://(.*?/.*?)/")
MIRROR_FILE_PATTERN = re.compile(r"mirror\+file:(/.*?)/pool")
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


REMOVING_FILE = _("{notice} Nala has removed {filename}")
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


@dataclass
class URL:  # pylint: disable=too-many-instance-attributes
	"""Representation of a URL and File for download."""

	uri: str
	size: int
	path: Path
	proto: str = ""
	hash_type: str = "sha256"
	hash: str = ""
	failed: bool = False
	no_hash: bool = False

	def filename(self) -> str:
		"""Return the final portion of the filename."""
		return self.path.name

	def dprint(self, received: str) -> None:
		"""Debug print the URL's hash status."""
		dprint(
			HASH_STATUS.format(
				filepath=self.path,
				hash_type=self.hash_type.upper(),
				expected=self.hash or "Not Provided",
				received=received,
				result=received == self.hash,
			)
		)

	@staticmethod
	def from_version(version: Version) -> URL:
		"""Return a URL from an Apt Version."""
		return URL(
			version.uri or "",
			version.size,
			# Have to run the filename through a path to get the last section
			ARCHIVE_DIR / get_pkg_name(version),
			"",
			*get_hash(version),
		)


class URLSet(List[URL]):
	"""Set of urls that are all expected to provide the same file."""

	def size(self) -> int:
		"""Return the 'on disk size' for the completed download."""
		return self[0].size

	def filename(self) -> str:
		"""Return the final portion of the filename."""
		return self[0].filename()

	def path(self) -> Path:
		"""Return the destinateion Path object."""
		return self[0].path

	def any_available(self) -> bool:
		"""Return True if there are any urls that haven't failed."""
		return any(not url.failed for url in self)

	def next_available(self) -> URL | None:
		"""Return URLs that haven't failed."""
		return next((url for url in self if not url.failed), None)

	@staticmethod
	def from_version(version: Version) -> URLSet:
		"""Return a URLSet from an Apt Version."""
		url_set = URLSet()
		for uri in version.uris:
			url_set.append(
				URL(
					uri,
					version.size,
					ARCHIVE_DIR / get_pkg_name(version),
					*get_hash(version),
				)
			)
		return url_set

	@staticmethod
	def from_str(uri: str) -> URLSet:
		"""Return a URLSet from str uri."""
		url_set = URLSet()
		hash_type = hashsum = ""
		# This means a hash must have been specified
		if len(split_url := uri.split(":")) > 2:
			if len(split_url) != 4:
				sys.exit(
					_(
						"{error} Improper Hash Syntax\n"
						"  Try '{url}"
						":sha256:510b0c4f8fc3e7bd7182b53c0b81c1a113bea9fd3c18219eac0e18e601dc8d17'"
					).format(error=ERROR_PREFIX, url=f"{split_url[0]}:{split_url[1]}")
				)
			proto, url, hash_type, hashsum = split_url
			# Rebuild the uri for downloading
			uri = f"{proto}:{url}"

		# We must get the response so we know what the filesize is.
		response = get(uri, follow_redirects=True)
		response.raise_for_status()
		dprint(response.headers)
		try:
			size = int(response.headers["content-length"])
		except KeyError:
			sys.exit(
				_(
					"{error} No content length in response from {url}\n"
					"  Ensure the URL points to a Debian Package"
				).format(error=ERROR_PREFIX, url=uri)
			)
		path = ARCHIVE_DIR / Path(uri).name
		if hash_type and hashsum:
			url_set.append(URL(uri, size, path, hash_type, hashsum))
		else:
			url_set.append(URL(uri, size, path, no_hash=True))
		return url_set


class Downloader:  # pylint: disable=too-many-instance-attributes
	"""Manage Package Downloads."""

	def __init__(self, pkgs: Sequence[URLSet]) -> None:
		"""Manage Package Downloads."""
		dprint("Downloader Initializing")
		self.total_pkgs: int = len(pkgs)
		self.total_data: int = 0
		self.count: int = 0
		self.live: Live
		self.last_completed: str = ""
		self.proxy: dict[HttpxUrl | str, HttpxUrl | str | Proxy | None] = {}
		self.failed: list[str] = []
		self.current: Counter[str] = Counter()
		self.fatal: bool = False
		self.exit: int | bool = False

		self.pkg_urls: Sequence[URLSet] = pkgs

		self.task = pkg_download_progress.add_task(
			"", total=sum(url.size() for url in pkgs)
		)

		self._set_proxy()
		dprint("Initialization Complete")

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
			# This code is for the common cases of configuring proxies
			if common_proxy := config.find(f"Acquire::{proto}::Proxy"):
				# If the proxy is set to direct or false we disable it
				if common_proxy.lower() not in ("direct", "false"):
					self.proxy[f"{proto}://"] = common_proxy

			# The remainder of code is for proxying specific repos. Such a configuration may look like
			# Acquire::http::Proxy::deb.volian.org "xxx:8087"
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
				# If direct or false, disable the proxy
				if value.lower() in ("direct", "false"):
					value = None
				self.proxy[f"{proto}://{key}"] = value

	async def _check_count(self, url: str) -> str:
		"""Check the url count and return if Nala should continue."""
		domain = ""
		if regex := URL_PATTERN.search(url):
			domain = regex.group(1)
			if self.current[domain] > 2:
				# Idk, but it doesn't work without the sleep.
				await sleep(0.01)
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

	async def _update_progress(self, len_data: int, failed: bool = False) -> None:
		"""Update download progress."""
		if failed:
			self.total_data -= len_data
			pkg_download_progress.reset(self.task, completed=self.total_data)
			return
		self.total_data += len_data
		pkg_download_progress.advance(self.task, advance=len_data)
		self.live.update(self._gen_table())

	async def start_download(self) -> bool:
		"""Start async downloads."""
		if not self.pkg_urls:
			return True
		with Live(get_renderable=self._gen_table, refresh_per_second=10) as self.live:
			async with AsyncClient(
				timeout=20,
				proxies=self.proxy,
				follow_redirects=True,
				# Custom user agent fixes some downloading issues
				# Caused by httpx default agent sometimes being blocked.
				headers={"user-agent": f"nala/{__version__}"},
			) as client:
				loop = asyncio.get_running_loop()
				tasks = (
					loop.create_task(self._init_download(client, url))
					for url in self.pkg_urls
				)

				# Setup handlers for Interrupts
				for signal_enum in (SIGINT, SIGTERM):
					exit_func = partial(self.interrupt, signal_enum, loop)
					loop.add_signal_handler(signal_enum, exit_func)

				return all(await gather(*tasks))

	async def _init_download(self, client: AsyncClient, urls: URLSet) -> None:
		"""Download pkgs."""
		while url := urls.next_available():
			for url in urls:
				if not (domain := await self._check_count(url.uri)):
					continue
				try:
					await self._download(client, url)

					post_download_check(url)

					# Download completed. Raise count, lower domain counter
					self.count += 1
					self.current[domain] -= 1
					self.last_completed = url.filename()
					self.live.update(self._gen_table())
					return

				except (HTTPError, OSError, FileDownloadError) as error:
					url.failed = True
					self.current[domain] -= 1
					self.download_error(error, urls)
					continue

	async def _download(self, client: AsyncClient, url: URL) -> None:
		"""Download and write package."""
		vprint(
			STARTING_DOWNLOAD_STATUS.format(
				starting_download=STARTING_DOWNLOAD,
				url=url.uri,
				size=unit_str(url.size).strip(),
			)
		)

		dest = PARTIAL_DIR / url.filename()
		second_attempt = False
		while True:
			total_data = 0
			hash_fun = hashlib.new(url.hash_type)
			try:
				async with client.stream("GET", url.uri) as response:
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
				dprint(f"Mirror Failed: {url.uri} {error}, will try again.")
				continue

			url.dprint(received := hash_fun.hexdigest())
			# URL is no hash when local debs are downloaded without
			# Specifying a hash
			if url.no_hash:
				vprint(f"Skipping hashsum for {url.filename()} as one wasn't provided")
				break

			if url.hash != received:
				dest.unlink()
				self.fatal = True
				await self._update_progress(total_data, failed=True)
				raise FileDownloadError(
					errno=FileDownloadError.ERRHASH,
					filename=dest.name,
					expected=f"{url.hash_type.upper()}: {url.hash}",
					received=f"{url.hash_type.upper()}: {received}",
				)
			break

	def download_error(self, error: DownloadErrorTypes, urls: URLSet) -> None:
		"""Handle download errors."""
		print_error(error)

		if not (next_url := urls.next_available()):
			# No need to say there are no mirrors available in the event there is only one.
			if len(urls) > 1:
				eprint(
					_("{error} No more mirrors available for {filename}").format(
						error=ERROR_PREFIX, filename=color(urls.filename(), "YELLOW")
					)
				)

			# Status error are fatal as apt_pkg is likely to fail with these as well
			if isinstance(error, HTTPStatusError):
				# 401 Unauthenticated should be allowed to pass to apt for authentication
				if error.response.status_code == 401:
					return
				self.fatal = True

			self.failed.append(urls.filename())
			return

		eprint(
			_("{notice} Trying next url: {url}").format(
				notice=NOTICE_PREFIX, url=next_url.uri
			)
		)


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

	# If 401 Unauthenticated we do not need to print anything
	# This is a secure repository like Ubuntu Pro. apt_pkg will pick it up
	if isinstance(error, HTTPStatusError) and error.response.status_code == 401:
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


def check_trusted(uri: str, candidate: Version) -> bool:
	"""Check if the candidate is trusted."""
	dprint(f"Checking trust of {candidate.package.name} ({candidate.version})")
	for (packagefile, _unused) in candidate._cand.file_list:
		if packagefile.site in uri and packagefile.archive != "now":
			indexfile = candidate.package._pcache._list.find_index(packagefile)
			dprint(f"{uri} = {indexfile and indexfile.is_trusted}")
			return bool(indexfile and indexfile.is_trusted)
	return False


def pre_download_check(url: URL) -> bool:
	"""Check if file exists, is correct, and run check hash."""
	dprint("Pre Download Package Check")

	if not url.path.exists():
		dprint(f"File Doesn't exist: {url.filename()}")
		return False

	if (size := url.path.stat().st_size) != url.size:
		dprint(f"File {url.filename()} has an unexpected size {size} != {url.size}")
		url.path.unlink()
		return False

	try:
		if not check_hash(url):
			dprint(f"Hash Checking has failed. Removing: {url.filename()}")
			url.path.unlink()
			return False
		dprint(f"Package doesn't require download: {url.filename()}")
		return True
	except OSError as err:
		eprint(_("Failed to check hash"), err)
		return False


def post_download_check(url: URL) -> bool:
	"""Check if file exists, is correct, and run check hash."""
	dprint("Post Download Package Check")

	source = PARTIAL_DIR / url.filename()
	try:
		dprint(f"Moving {source} -> {url.path}")
		source.rename(url.path)
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

	if not url.path.exists():
		dprint(f"File Doesn't exist: {url.filename()}")
		raise FileDownloadError(
			errno=FileDownloadError.ENOENT,
			filename=url.filename(),
		)

	if (size := url.path.stat().st_size) != url.size:
		dprint(f"File {url.filename()} has an unexpected size {size} != {url.size}")
		url.path.unlink()
		raise FileDownloadError(
			errno=FileDownloadError.ERRSIZE,
			filename=url.filename(),
			expected=f"{url.size}",
			received=f"{size}",
		)

	vprint(
		DOWNLOAD_COMPLETE_STATUS.format(
			download_complete=DOWNLOAD_COMPLETE, url=url.uri
		)
	)

	# Hash was checked while downloading. We can just call it good.
	return True


def check_hash(url: URL) -> bool:
	"""Check hash value."""
	hash_fun = hashlib.new(url.hash_type)
	with url.path.open("rb") as file:
		while data := file.read(4096):
			hash_fun.update(data)

	received = hash_fun.hexdigest()
	url.dprint(received)
	return received == url.hash


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


def filter_local_repo(pkgs: Iterable[Package]) -> list[URLSet]:
	"""Filter any local repository packages.

	This will check if the packages are coming from a local repository.

	If a package is from a local repo it will move it into the archive directory.

	Lastly it will checksum all packages that exist on the system
	and return a list of packages that need to be downloaded.
	"""
	file_uris: list[str] = []
	untrusted: list[str] = []
	for pkg in pkgs:
		# At this point anything that makes it will have a candidate
		if not pkg.candidate or pkg.marked_delete:
			continue

		# Check through the uris and see if there are any file paths
		for uri in pkg.candidate.uris:
			if not uri.startswith("file:"):
				continue
			# We must check trust at this point
			if not check_trusted(uri, pkg.candidate):
				untrusted.append(pkg.candidate.filename)
			# All was well so append our uri and break for the next pkg
			file_uris.append(uri)
			break

	# Exit with an error if there are unauthenticated packages
	# This can proceed if overridden by configuration
	if untrusted:
		untrusted_error(untrusted)

	# Copy the local repo debs into the archive directory
	# This is a must so `apt` knows about it and we can check the hash
	for file in file_uris:
		dprint("Moving files from local repository")
		src = Path(file.lstrip("file:"))
		dest = ARCHIVE_DIR / src.name
		# Make sure that the source exists and the destination does not
		if src.is_file() and not dest.is_file():
			# Move the file to the archive directory.
			# We're allowed to do this silently because hashsum comes later
			dprint(f"{src} => {shutil.copy2(src, dest)}")

	# Return the list of packages that should be downloaded
	return versions_to_urls(
		pkg.candidate
		for pkg in pkgs
		# Don't download packages that already exist
		if pkg.candidate
		and not pkg.marked_delete
		and not pre_download_check(URL.from_version(pkg.candidate))
	)


def versions_to_urls(versions: Iterable[Version]) -> list[URLSet]:
	"""Convert Apt Versions into urls for the downloader."""
	urls: list[URLSet] = []
	untrusted: list[str] = []
	mirrors: dict[str, list[str]] = {}
	for version in versions:
		url_set = URLSet()
		for uri in filter_uris(version, mirrors, untrusted):
			hash_type, hashsum = get_hash(version)
			url_set.append(
				URL(
					uri,
					version.size,
					# Have to run the filename through a path to get the last section
					ARCHIVE_DIR / get_pkg_name(version),
					hash_type=hash_type,
					hash=hashsum,
				)
			)
		urls.append(url_set)

	if untrusted:
		untrusted_error(untrusted)

	return urls


def filter_uris(
	candidate: Version, mirrors: dict[str, list[str]], untrusted: list[str]
) -> Generator[str, None, None]:
	"""Filter uris into usable urls."""
	for uri in candidate.uris:
		# Sending a file path through the downloader will cause it to lock up
		# These have already been handled before the downloader runs.
		if uri.startswith("file:"):
			continue
		if not check_trusted(uri, candidate):
			untrusted.append(color(candidate.package.name, "RED"))
		# Regex to check if we're using mirror://
		if regex := MIRROR_PATTERN.search(uri):
			set_mirrors_txt(domain := regex.group(1), mirrors)
			yield from (
				link + candidate.filename
				for link in mirrors[domain]
				if not link.startswith("#")
			)
			continue
		# Regex to check if we're using mirror+file:/
		if regex := MIRROR_FILE_PATTERN.search(uri):
			if regex.group(1) not in mirrors:
				mirrors[regex.group(1)] = (
					Path(regex.group(1)).read_text(encoding="utf-8").splitlines()
				)

			yield from (
				f"{link}/{candidate.filename}"
				for link in mirrors[regex.group(1)]
				if not link.startswith("#")
			)
			continue
		yield uri


def set_mirrors_txt(domain: str, mirrors: dict[str, list[str]]) -> None:
	"""Check if user has mirrors:// and handle accordingly."""
	if domain not in mirrors:
		url = f"http://{domain}"
		try:
			mirrors[domain] = get(url, follow_redirects=True).text.splitlines()
		except HTTPError:
			sys.exit(
				_("{error} unable to connect to {url}").format(
					error=ERROR_PREFIX, url=url
				)
			)


def download(downloader: Downloader) -> None:
	"""Run downloads and check for failures.

	Does not return if in Download Only mode.
	"""
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


def download_pkgs(pkgs: Iterable[Package]) -> None:
	"""Download package from a list of pkgs."""
	download(
		Downloader(
			# Start the larger files first, as they take the longest
			sorted(
				filter_local_repo(pkgs),
				key=lambda url: url.size(),
				reverse=True,
			)
		)
	)


def download_strings(urls: Iterable[str]) -> None:
	"""Download packages from a list of urls.

	This function assumes that each URL is its own package
	and will create a URLSet for each str url in the iterable
	"""
	download(Downloader([URLSet.from_str(url) for url in urls]))
