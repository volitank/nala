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
"""Functions for the Nala Install command."""
from __future__ import annotations

import contextlib
import fnmatch
import hashlib
import sys
from pathlib import Path
from shutil import which
from typing import Iterable, List, Sequence, cast

import apt_pkg
from apt.cache import FetchFailedException, LockFailedException
from apt.package import BaseDependency, Dependency, Package
from apt_pkg import DepCache, Error as AptError, get_architectures
from httpx import HTTPError, head

from nala import _, color, color_version
from nala.cache import Cache
from nala.constants import (
	ARCHIVE_DIR,
	DPKG_LOG,
	ERROR_PREFIX,
	NALA_DIR,
	NALA_TERM_LOG,
	NOTICE_PREFIX,
	REBOOT_PKGS,
	REBOOT_REQUIRED,
	WARNING_PREFIX,
	CurrentState,
)
from nala.debfile import NalaBaseDep, NalaDebPackage, NalaDep
from nala.downloader import (
	URL,
	Downloader,
	URLSet,
	download,
	download_pkgs,
	print_error,
)
from nala.dpkg import DpkgLive, InstallProgress, OpProgress, UpdateProgress, notice
from nala.error import (
	BrokenError,
	ExitCode,
	apt_error,
	essential_error,
	local_deb_error,
	print_dpkg_errors,
)
from nala.history import write_history
from nala.options import arguments
from nala.rich import ELLIPSIS, Text, dpkg_progress, from_ansi
from nala.summary import print_update_summary
from nala.utils import (
	DelayedKeyboardInterrupt,
	NalaPackage,
	PackageHandler,
	ask,
	dprint,
	eprint,
	get_date,
	get_pkg_version,
	pkg_installed,
	term,
	unauth_ask,
	vprint,
)

# pylint: disable=too-many-lines


def auto_remover(cache: Cache, nala_pkgs: PackageHandler, config: bool = False) -> None:
	"""Handle auto removal of packages."""
	dprint("Starting Auto Remover")
	if not arguments.auto_remove and arguments.command not in (
		"autoremove",
		"autopurge",
	):
		dprint("Packages will not be autoremoved")
		nala_pkgs.not_needed = [
			NalaPackage(pkg.name, pkg.installed.version, pkg.installed.installed_size)
			for pkg in cache
			if pkg.installed and not pkg.marked_delete and pkg.is_auto_removable
		]
		return

	dprint("Auto-Removing")
	with cache.actiongroup():  # type: ignore[attr-defined]
		# Recurse 10 levels if we're installing .debs to make sure that all depends are safe
		deps = recurse_deps(nala_pkgs.local_debs, levels=10, installed=False)
		for pkg in cache:
			if pkg.is_installed and not pkg.marked_delete and pkg.is_auto_removable:
				if pkg in deps:
					dprint(f"Dependency: ['{pkg.name}'] Protected from removal")
					continue
				# We don't have to autofix while autoremoving
				pkg.mark_delete(auto_fix=False, purge=arguments.is_purge())
				nala_pkgs.autoremoved.add(pkg.name)

			elif config and not pkg.is_installed and pkg.has_config_files:
				vprint(
					_("Purging configuration files for {package}").format(
						package=color(pkg.name, "RED")
					)
				)
				pkg.mark_delete(auto_fix=False, purge=arguments.is_purge())
				nala_pkgs.autoremoved.add(pkg.name)

	dprint(f"Pkgs marked by autoremove: {nala_pkgs.autoremoved}")


def recurse_deps(
	pkgs: Iterable[NalaDebPackage] | Iterable[Package],
	levels: int = 1,
	installed: bool = False,
) -> set[Package]:
	"""Return the installed dependency packages.

	This function can be used recursively.

	Example for recursing deps 2 levels, returning only those installed::

	        deps = installed_deps(list[Package], 2, installed=True)

	Args::

	        pkgs (list[NalaDebPackage] | list[Package]):  list of package objects.
	        recurse (int): How many levels to traverse dependencies. Default is 1.
	        installed (bool): Whether to grab dependencies that are installed or all. Default False.
	"""
	if not pkgs:
		# Return an empty list if we didn't get packages passed.
		return set()
	total_deps: set[Package] = set()
	for _ in range(levels):
		dep_list: set[Package] = set()
		for dpkg in pkgs:
			dependencies = get_dep_type(dpkg, installed)
			for deps in dependencies:
				# deps len greater than 1 are or_deps
				if len(deps) > 1:
					for ndep in deps:
						dep_list |= get_dep_pkgs(ndep, installed)
					continue
				dep_list |= get_dep_pkgs(deps[0], installed)
		total_deps |= dep_list
		pkgs = dep_list
	dprint(
		f"Recurse Levels: {levels}, Recursive List Size: {len(total_deps)}, "
		f"Recurse Type: {'Installed' if installed else 'All Packages'}"
	)
	return total_deps


def get_dep_pkgs(
	ndep: NalaBaseDep | BaseDependency, installed: bool = False
) -> set[Package]:
	"""Return the packages of the specified dep that we are to use."""
	target_versions = (
		ndep.installed_target_versions if installed else ndep.target_versions
	)
	return {version.package for version in target_versions}


def get_dep_type(
	dpkg: NalaDebPackage | Package, installed: bool = False
) -> list[Dependency] | list[NalaDep]:
	"""Return the installed or candidate dependencies."""
	if isinstance(dpkg, Package):
		# We know it is installed as we check outside this function
		if installed and dpkg.installed:
			return dpkg.installed.dependencies
		if not installed and dpkg.candidate:
			return dpkg.candidate.dependencies
		return cast(List[Dependency], [])
	return dpkg.dependencies


def fix_excluded(protected: set[Package], is_upgrade: Iterable[Package]) -> list[str]:
	"""Find and optionally fix packages that need protecting."""
	eprint(
		_("{notice} Selected packages cannot be excluded from upgrade safely.").format(
			notice=NOTICE_PREFIX
		)
	)
	new_pkg = set()
	old_pkg = set()
	for pkg in protected:
		old_pkg.add(pkg.name)
		if not pkg.candidate:
			continue
		for deps in pkg.candidate.dependencies:
			for base_dep in deps:
				if base_dep.target_versions:
					dep_pkg = base_dep.target_versions[0].package
					if (
						dep_pkg in is_upgrade
						and dep_pkg.marked_install
						or dep_pkg.marked_upgrade
					):
						new_pkg.add(dep_pkg.name)
	if not new_pkg:
		eprint(
			_(
				"{error} Unable to calculate how to protect the selected packages"
			).format(error=ERROR_PREFIX)
		)
		sys.exit(_("{error} You have held broken packages").format(error=ERROR_PREFIX))
	eprint(
		_("{notice} The following packages need to be protected as well:").format(
			notice=NOTICE_PREFIX
		)
	)
	eprint(
		f"  {' '.join(color(name, 'YELLOW') for name in sorted(new_pkg) if name not in old_pkg)}\n"
	)
	return sorted(new_pkg | old_pkg)


def hook_exists(key: str, pkg_names: set[str]) -> str:
	"""Return True if the hook file exists on the system."""
	if "*" in key and (globbed := fnmatch.filter(pkg_names, key)):
		return globbed[0]
	return key if key == "hook" or key in pkg_names else ""


def parse_hook_args(
	pkg: str, hook: dict[str, str | list[str]], cache: Cache
) -> list[str]:
	"""Parse the arguments for the advanced hook."""
	invalid: list[str] = []
	cmd = cast(str, hook.get("hook", "")).split()
	if args := cast(List[str], hook.get("args", [])):
		arg_pkg = cache[pkg]
		for arg in args:
			# See if they are valid base package attributes
			if arg in ("name", "fullname"):
				cmd.append(getattr(arg_pkg, arg))
				continue

			# Convert simple args to candidate args
			if arg in ("version", "architecture"):
				arg = f"candidate.{arg}"

			# Otherwise they could be a specific version argument
			# arg = "candidate.arch"
			if (
				arg.startswith(("candidate.", "installed."))
				and len(arg_split := arg.split(".")) > 1
				and arg_split[1] in ("version", "architecture")
			):
				version = (
					arg_pkg.candidate
					if arg_split[0] == "candidate"
					else arg_pkg.installed
				)
				cmd.append(getattr(version, arg_split[1]) if version else "None")
				continue

			# If none of these matched then the requested argument is invalid.
			invalid.append(color(arg, "YELLOW"))

	if invalid:
		sys.exit(
			_("{error} The following hook arguments are invalid: {args}").format(
				error=ERROR_PREFIX, args=", ".join(invalid)
			)
		)
	return cmd


def check_hooks(pkg_names: set[str], cache: Cache) -> None:
	"""Check that the hook paths exist before trying to run anything."""
	bad_hooks: dict[str, list[str]] = {
		"PreInstall": [],
		"PostInstall": [],
	}
	for hook_type, hook_list in bad_hooks.items():
		for key, hook in arguments.config.get_hook(hook_type).items():
			if pkg := hook_exists(key, pkg_names):
				if isinstance(hook, dict):
					# Print a pretty debug message for the hooks
					pretty = [(f"{key} = {value},\n") for key, value in hook.items()]
					dprint(
						f"{hook_type} {{\n"
						f"{(indent := '    ')}Key: {key}, Hook: {{\n"
						f"{indent*2}{f'{indent*2}'.join(pretty)}{indent}}}\n}}"
					)
					cmd = parse_hook_args(pkg, hook, cache)
				else:
					dprint(f"{hook_type} {{ Key: {key}, Hook: {hook} }}")
					cmd = hook.split()

				# Check to make sure we can even run the hook
				if not which(cmd[0]):
					hook_list.append(color(cmd, "YELLOW"))
					continue

				dprint(f"Hook Command: {' '.join(cmd)}")
				if hook_type == "PreInstall":
					arguments.config.apt.set("DPkg::Pre-Invoke::", " ".join(cmd))
				elif hook_type == "PostInstall":
					arguments.config.apt.set("DPkg::Post-Invoke::", " ".join(cmd))

	# If there are no bad hooks we can continue with the installation
	if not bad_hooks["PreInstall"] + bad_hooks["PostInstall"]:
		return

	# There are bad hooks, so we should exit as to not mess anything up
	for hook_type, hook_list in bad_hooks.items():
		if hook_list:
			eprint(
				_("{error} The following {hook_type} commands cannot be found.").format(
					error=ERROR_PREFIX, hook_type=hook_type
				)
			)
			eprint(f"  {', '.join(hook_list)}")
	sys.exit(1)


def commit_pkgs(cache: Cache, nala_pkgs: PackageHandler) -> None:
	"""Commit the package changes to the cache."""
	dprint("Commit Pkgs")
	task = dpkg_progress.add_task("", total=nala_pkgs.dpkg_progress_total())
	check_hooks({pkg.name for pkg in nala_pkgs.all_pkgs()}, cache)

	with DpkgLive(install=True) as live:
		with open(DPKG_LOG, "w", encoding="utf-8") as dpkg_log:
			with open(NALA_TERM_LOG, "a", encoding="utf-8") as term_log:
				term_log.write(
					_("Log Started: [{date}]").format(date=get_date()) + "\n"
				)
				if arguments.raw_dpkg:
					live.stop()
				config_purge = tuple(
					pkg.name
					for pkg in nala_pkgs.autoremove_config + nala_pkgs.delete_config
				)
				install = InstallProgress(dpkg_log, term_log, live, task, config_purge)
				update = UpdateProgress(live)
				cache.commit_pkgs(install, update)
				if nala_pkgs.local_debs:
					cache.commit_pkgs(install, update, nala_pkgs.local_debs)
				term_log.write(
					_("Log Ended: [{date}]").format(date=get_date()) + "\n\n"
				)
		# If we made it this far just set the total to 100%.
		dpkg_progress.reset(task)
		dpkg_progress.advance(task, advance=nala_pkgs.dpkg_progress_total())
		live.scroll_bar(rerender=True)


def get_changes(cache: Cache, nala_pkgs: PackageHandler, operation: str) -> None:
	"""Get packages requiring changes and process them."""
	cache.purge_removed()
	pkgs = sorted(cache.get_changes(), key=sort_pkg_name)
	if not NALA_DIR.exists():
		NALA_DIR.mkdir()

	if operation not in ("upgrade", "remove"):
		if not arguments.install_recommends:
			get_extra_pkgs(
				"Recommends", pkgs + nala_pkgs.local_debs, nala_pkgs.recommend_pkgs  # type: ignore[operator]
			)
		if not arguments.install_suggests:
			get_extra_pkgs(
				"Suggests", pkgs + nala_pkgs.local_debs, nala_pkgs.suggest_pkgs  # type: ignore[operator]
			)

	check_work(pkgs, nala_pkgs, operation)

	if pkgs or nala_pkgs.local_debs or nala_pkgs.configure_pkgs:
		check_essential(pkgs)
		sort_pkg_changes(pkgs, nala_pkgs)
		print_update_summary(nala_pkgs, cache)
		check_term_ask()

	# Enable verbose and raw_dpkg if we're piped.
	if not term.can_format():
		arguments.verbose = True
		arguments.raw_dpkg = True
	# If we're in Raw_Dpkg we can restore the locale as Nala doesn't handle the output
	if arguments.raw_dpkg:
		term.restore_locale()

	download_pkgs(pkgs)
	write_history(cache, nala_pkgs, operation)
	start_dpkg(cache, nala_pkgs)


def start_dpkg(cache: Cache, nala_pkgs: PackageHandler) -> None:
	"""Start dpkg."""
	try:
		# Set Use-Pty to False. This makes Sigwinch signals accepted by dpkg.
		apt_pkg.config.set("Dpkg::Use-Pty", "0")
		commit_pkgs(cache, nala_pkgs)
	# Catch system error because if dpkg fails it'll throw this
	except (apt_pkg.Error, SystemError) as error:
		apt_error(error)
	except FetchFailedException as error:
		# We have already printed this error likely. but just in case
		# We write it to the dpkg_log so at least we'll know about it.
		with open(DPKG_LOG, "a", encoding="utf-8") as file:
			file.write("FetchedFailedException:\n")
			file.write(f"{error}")
		eprint(_("{error} Fetching packages has failed!").format(error=ERROR_PREFIX))
		sys.exit(1)
	except KeyboardInterrupt:
		eprint(_("Exiting due to SIGINT"))
		sys.exit(ExitCode.SIGINT)
	finally:
		term.restore_mode()
		# If dpkg quits for any reason we lose the cursor
		if term.can_format():
			term.write(term.SHOW_CURSOR)

		print_dpkg_errors()
		print_notices(notice)
		if need_reboot():
			print(_("{notice} A reboot is required.").format(notice=NOTICE_PREFIX))
	print(color(_("Finished Successfully"), "GREEN"))


def install_local(nala_pkgs: PackageHandler, cache: Cache) -> None:
	"""Mark the depends for local debs to be installed.

	Dependencies that are marked will be marked auto installed.

	Returns local_names to be printed in the transaction summary.
	"""
	failed: list[NalaDebPackage] = []
	for pkg in nala_pkgs.local_debs[:]:
		if not pkg.check(allow_downgrade=True):
			failed.append(pkg)
			nala_pkgs.local_debs.remove(pkg)
			continue

		if not check_local_version(pkg, nala_pkgs):
			nala_pkgs.install_pkgs.append(
				NalaPackage(
					pkg.pkgname,
					pkg._sections["Version"],
					pkg.installed_size(),
				)
			)

		depends = pkg.dependencies
		extra_pkgs = (
			(arguments.install_recommends, pkg.get_dependencies("Recommends")),
			(arguments.install_suggests, pkg.get_dependencies("Suggests")),
		)

		# Check to see if we need to install any recommends or suggests.
		for check, extra_deps in extra_pkgs:
			if not check:
				continue

			for dep in extra_deps:
				if dep[0].name in cache:
					cache[dep[0].name].mark_install(auto_fix=arguments.fix_broken)
					depends.append(dep)

		satisfy_notice(pkg, depends)

	if failed:
		BrokenError(cache, failed).broken_install()


def satisfy_notice(pkg: NalaDebPackage, depends: list[NalaDep]) -> None:
	"""Print a notice of how to satisfy the packages dependencies."""
	fixer: list[str] = []

	for dep in depends:
		fixer.extend(
			color(ppkg.name, "GREEN")
			for base_dep in dep
			if (target := list(base_dep.target_versions))
			and (ppkg := target[0].package).marked_install
		)

	if fixer:
		print(
			_("{notice} The following will be installed to satisfy {package}:").format(
				notice=NOTICE_PREFIX, package=color(pkg.name, "GREEN")
			)
		)
		print(f"  {', '.join(fixer)}")


def check_local_version(pkg: NalaDebPackage, nala_pkgs: PackageHandler) -> bool:
	"""Check if the version installed is better than the .deb.

	Return True if we've added to a package list, False if not.

	VERSION_NONE = 0
	VERSION_OUTDATED = 1
	VERSION_SAME = 2
	VERSION_NEWER = 3
	"""
	if pkg_compare := pkg.compare_to_version_in_cache():
		cache_pkg = pkg._cache[pkg.pkgname]
		dprint(f"Comparing cache versions of: {pkg.pkgname}")
		# Example filename ../scar-archive/nala_0.5.0_amd64.deb
		assert pkg.filename
		dprint(f"Filename: {(filename := pkg.filename.split('/')[-1])}")
		dprint(f"Cache pkg: {cache_pkg.fullname}")
		if pkg_compare == pkg.VERSION_SAME and cache_pkg.is_installed:
			dprint(f"{filename} is the same version as the installed pkg")
			nala_pkgs.reinstall_pkgs.append(
				NalaPackage(
					pkg.pkgname,
					pkg._sections["Version"],
					pkg.installed_size(),
					pkg_installed(cache_pkg).version,
				)
			)

			if (
				pkg.compare_to_version_in_cache(use_installed=False)
				== pkg.VERSION_OUTDATED
			):
				if not cache_pkg.candidate:
					return True
				color_name = color(cache_pkg.name, "GREEN")
				print(
					_(
						"{notice} Newer version {package} {version} exists in the cache.\n"
						"You should consider using `{command}`"
					).format(
						notice=NOTICE_PREFIX,
						package=color_name,
						version=color_version(cache_pkg.candidate.version),
						command=f"{color('nala install')} {color_name}",
					)
				)
			return True

		if pkg_compare == pkg.VERSION_OUTDATED:
			dprint(f"{pkg.filename} is an older version than the installed pkg")
			nala_pkgs.downgrade_pkgs.append(
				NalaPackage(
					pkg.pkgname,
					pkg._sections["Version"],
					pkg.installed_size(),
					pkg_installed(cache_pkg).version,
				)
			)
			return True
		if pkg_compare == pkg.VERSION_NEWER and cache_pkg.is_installed:
			dprint(f"{pkg.filename} is a newer version than the installed pkg")
			nala_pkgs.upgrade_pkgs.append(
				NalaPackage(
					pkg.pkgname,
					pkg._sections["Version"],
					pkg.installed_size(),
					pkg_installed(cache_pkg).version,
				)
			)
			return True
	return False


def prioritize_local(
	deb_pkg: NalaDebPackage, cache_name: str, pkg_names: list[str]
) -> None:
	"""Print a notice of prioritization and remove the pkg name from list."""
	assert deb_pkg.filename
	print(
		_("{notice} {deb} has taken priority over {package} from the cache.").format(
			notice=NOTICE_PREFIX,
			deb=color(deb_pkg.filename.split("/")[-1], "GREEN"),
			package=color(cache_name, "YELLOW"),
		)
	)
	pkg_names.remove(cache_name)


def get_url_size(url: str) -> int:
	"""Get the URL Header and check for content length."""
	# We must get the headers so we know what the filesize is.
	response = head(url, follow_redirects=True)
	response.raise_for_status()
	dprint(response.headers)

	try:
		return int(response.headers["content-length"])
	except KeyError:
		sys.exit(
			_(
				"{error} No content length in response from {url}\n"
				"  Ensure the URL points to a Debian Package"
			).format(error=ERROR_PREFIX, url=url)
		)


def split_url(url_string: str, cache: Cache) -> URLSet:
	"""Split the URL and try to determine the hash."""
	dprint(url_split := url_string.split(":"))

	# http
	proto = url_split[0]
	# //deb.debian.org/debian/pool/main/n/neofetch/neofetch_7.1.0-2_all.deb
	body = url_split[1]

	# neofetch_7.1.0-2_all.deb
	dprint(f"Filename: {(filename := body.split('/').pop())}")

	# Initialize a URL
	url = URL(
		f"{proto}:{body}",
		get_url_size(f"{proto}:{body}"),
		ARCHIVE_DIR / filename,
		proto,
	)

	pkg_attrs = filename.split("_")
	# ["neofetch", "7.1.0-2", "all"]
	dprint(f"Package Name: {(pkg_attrs[0])}\n")

	try:
		hash_or_type = url_split[2]
	# IndexError Occurs because they did not specify a hash
	except IndexError:
		eprint(
			_("{notice} {filename} can't be hashsum verified.").format(
				notice=NOTICE_PREFIX, filename=filename
			)
		)

		if not unauth_ask(_("Do you want to continue?")):
			sys.exit(_("Abort."))

		url.no_hash = True
		return URLSet([url])

	# sha512 d500faf8b2b9ee3a8fbc6a18f966076ed432894cd4d17b42514ffffac9ee81ce
	# 945610554a11df24ded152569b77693c57c7967dd71f644af3066bf79a923bfe
	#
	# sha256 a694f44fa05fff6d00365bf23217d978841b9e7c8d7f48e80864df08cebef1a8
	# md5 b9ef863f210d170d282991ad1e0676eb
	# sha1 d1f34ed00dea59f886b9b99919dfcbbf90d69e15

	# Length of the hex digests
	len_map = {
		128: "sha512",
		64: "sha256",
		32: "md5sum",
		40: "sha1",
	}

	# Clear the hash_type
	url.hash_type = ""
	# Attempt to autodetect the hash type based on the len of the hash
	if hash_type := len_map.get(len(hash_or_type)):
		url.hash_type = hash_type
		url.hash = hash_or_type
		vprint(f"Automatically Selecting {url.hash_type}: {url.hash}")

	# If it doesn't match then it must be specified
	else:
		url.hash_type = hash_or_type

		try:
			url.hash = url_split[3]
			# This is just testing to ensure it's supported
			hashlib.new(url.hash_type)

			# If the Type is known we can check the length of the hash to ensure that it's proper
			if (
				url.hash_type in len_map.values()
				and url.hash_type != len_map[len(url.hash)]
			):
				sys.exit(
					_("{error} Hash does not match the '{hash_type}' Length").format(
						error=ERROR_PREFIX, hash_type=color(url.hash_type, "YELLOW")
					)
				)

		except IndexError:
			sys.exit(
				_("{error} Hash Type '{hash_type}' specified with no hash").format(
					error=ERROR_PREFIX, hash_type=color(url.hash_type, "YELLOW")
				)
			)

		except ValueError:
			sys.exit(
				_("{error} Hash Type '{hash_type}' is unsupported").format(
					error=ERROR_PREFIX, hash_type=color(url.hash_type, "YELLOW")
				)
			)

	# Everything hashed out, lets check for any extra URI's we can add
	dprint(url)
	url_set = URLSet([url])

	# Check to see if our package is in the cache
	if pkg_attrs[0] in cache:
		dprint("Package found in the cache")
		pkg = cache[pkg_attrs[0]]
		for ver in pkg.versions:
			hash_list = ver._records.hashes
			with contextlib.suppress(KeyError):
				if url.hash == hash_list.find(url.hash_type).hashvalue:
					dprint("Package Hash Found in the cache. Adding URIs")
					url_set.append(URL.from_version(ver))

	# You can check the versions with this. I don't know if it's useful yet
	# pylint: disable=line-too-long
	# if (cmp := apt_pkg.version_compare(version, ver.version)) > 0:
	# 	eprint("A nice little upgrade!")
	# elif cmp < 0:
	# 	eprint(_("{notice} Woah are you planning on downgrading this package?").format(notice=NOTICE_PREFIX))
	# else:
	# 	print("The Versions are identical")
	return url_set


def split_local(
	pkg_names: list[str], cache: Cache, local_debs: list[NalaDebPackage]
) -> list[str]:
	"""Split pkg_names into either Local debs, regular install or they don't exist."""
	not_exist: list[str] = []
	download_debs = []
	if urls := [name for name in pkg_names if name.startswith(("http://", "https://"))]:
		print(f"Checking Urls{ELLIPSIS}")
		for url in urls:
			try:
				vprint(f"Verifying {url}")
				url_set = split_url(url, cache)
			except HTTPError as error:
				print_error(error)
				sys.exit(1)

			download_debs.append(url_set)
			pkg_names.remove(url)
			pkg_names.append(f"{url_set.path()}")

	# .deb packages have to be downloaded before anything else in order to determine dependencies
	if download_debs:
		download(Downloader(download_debs))

	for name in pkg_names[:]:
		if ".deb" in name or "/" in name:
			path = Path(name)
			if not path.exists():
				not_exist.append(name)
				pkg_names.remove(name)
				continue
			try:
				local_debs.append(deb_pkg := NalaDebPackage(name, cache))
			except AptError as error:
				local_deb_error(error, name)

			if deb_pkg.pkgname in pkg_names and deb_pkg.pkgname in cache:
				prioritize_local(deb_pkg, deb_pkg.pkgname, pkg_names)
			for arch in get_architectures():
				if (
					arch_pkg := f"{deb_pkg.pkgname}:{arch}"
				) in pkg_names and arch_pkg in cache:
					prioritize_local(deb_pkg, arch_pkg, pkg_names)

			pkg_names.remove(name)
			continue
	return not_exist


def package_manager(pkg_names: list[str], cache: Cache, remove: bool = False) -> bool:
	"""Manage installation or removal of packages."""
	with cache.actiongroup():  # type: ignore[attr-defined]
		for pkg_name in pkg_names:
			if pkg_name in cache:
				pkg = cache[pkg_name]
				try:
					if remove:
						if pkg.installed or (
							pkg.has_config_files and arguments.is_purge()
						):
							pkg.mark_delete(
								auto_fix=arguments.fix_broken,
								purge=arguments.is_purge(),
							)
							dprint(f"Marked Remove: {pkg.name}")
						continue
					if not pkg.installed or pkg.marked_downgrade:
						pkg.mark_install(auto_fix=arguments.fix_broken)
						dprint(f"Marked Install: {pkg.name}")
					elif pkg.is_upgradable:
						pkg.mark_upgrade()
						dprint(f"Marked upgrade: {pkg.name}")
				except AptError as error:
					if (
						"broken packages" not in f"{error}"
						and "held packages" not in f"{error}"
					):
						raise error from error
					return False
	return True


def set_candidate_versions(
	pkg_names: list[str], cache: Cache
) -> tuple[list[str], bool]:
	"""Set the version to be installed."""
	not_found: list[str] = []
	failed = False
	for name in pkg_names[:]:
		if "=" not in name:
			continue
		pkg_name, version = name.split("=")

		if pkg_name not in cache:
			not_found.append(name)
			pkg_names.remove(name)
			continue

		pkg = cache[pkg_name]
		found = False
		for ver in pkg.versions:
			if ver.version == version:
				pkg.candidate = ver
				pkg_names.remove(name)
				pkg_names.append(pkg_name)
				found = True
				# Break because the same version could exist multiple times
				# Example, nala is in both Sid and Volian repository
				break

		if found:
			continue
		failed = True
		eprint(
			_("{error} Version {version} not found for package {package}").format(
				error=ERROR_PREFIX,
				version=color_version(version),
				package=color(pkg_name, "GREEN"),
			)
		)
	return not_found, failed


# pylint: disable=import-outside-toplevel, cyclic-import
def check_state(cache: Cache, nala_pkgs: PackageHandler) -> None:
	"""Check if pkg needs to be configured so we can show it."""
	from nala.nala import _fix_broken

	if cache.broken_count and arguments.fix_broken:
		_fix_broken(cache)
		sys.exit()
	for raw_pkg in cache._cache.packages:
		if raw_pkg.current_state in (
			CurrentState.HALF_CONFIGURED,
			CurrentState.UNPACKED,
		):
			pkg = cache[raw_pkg.name]
			if pkg.installed:
				nala_pkgs.configure_pkgs.append(
					NalaPackage(
						pkg.name, pkg.installed.version, pkg.installed.installed_size
					)
				)


def get_extra_pkgs(  # pylint: disable=too-many-branches
	extra_type: str,
	pkgs: Sequence[Package | NalaDebPackage],
	npkg_list: list[NalaPackage | list[NalaPackage]],
) -> None:
	"""Get Recommended or Suggested Packages."""
	dprint(f"Getting `{extra_type}` Packages")
	or_name = []
	for pkg in pkgs:
		if isinstance(pkg, Package):
			recommends: list[Dependency] | list[NalaDep]
			if not pkg.marked_install or not pkg.candidate:
				continue
			if not (recommends := pkg.candidate.get_dependencies(extra_type)):
				continue
		# Then The package must be a NalaDebPackage
		elif not (recommends := pkg.get_dependencies(extra_type)):
			continue

		for dep in recommends:
			# We don't need to show this if the extra is satisfied
			if dep.installed_target_versions:
				continue
			if len(dep) == 1:
				if not dep.target_versions:
					npkg_list.append(NalaPackage(dep[0].name, _("Virtual Package"), 0))
					continue
				ver = dep.target_versions[0]
				# We don't need to show if it's to be installed
				if ver.package.marked_install:
					continue
				npkg_list.append(NalaPackage(ver.package.name, ver.version, ver.size))
				continue
			or_deps = []
			for base_dep in dep:
				if not base_dep.target_versions:
					if base_dep.name in or_name:
						continue
					or_name.append(base_dep.name)
					or_deps.append(NalaPackage(base_dep.name, _("Virtual Package"), 0))
					continue
				ver = base_dep.target_versions[0]
				# We don't need to show if it's to be installed
				if ver.package.name in or_name or ver.package.marked_install:
					continue
				or_name.append(ver.package.name)
				or_deps.append(NalaPackage(ver.package.name, ver.version, ver.size))
			if len(or_deps) == 1:
				npkg_list.extend(or_deps)
				continue
			if or_deps:
				npkg_list.append(or_deps)


def check_broken(
	pkg_names: list[str], cache: Cache, remove: bool = False
) -> tuple[list[Package], list[str], bool]:
	"""Check if packages will be broken."""
	broken_count = 0
	broken: list[Package] = []
	depcache = cache._depcache

	not_found, failed = set_candidate_versions(pkg_names, cache)
	with cache.actiongroup():  # type: ignore[attr-defined]
		for pkg_name in pkg_names[:]:
			if pkg_name not in cache:
				not_found.append(pkg_name)
				continue

			pkg = cache[pkg_name]
			mark_pkg(pkg, depcache, remove=remove)
			if depcache.broken_count > broken_count and arguments.fix_broken:
				broken.append(pkg)
				broken_count += 1
	return broken, not_found, failed


def mark_pkg(pkg: Package, depcache: DepCache, remove: bool = False) -> bool:
	"""Mark Packages in depcache for broken checks."""
	if remove:
		if not pkg.installed and not (pkg.has_config_files and arguments.is_purge()):
			eprint(
				_("{notice} {package} is not installed").format(
					notice=NOTICE_PREFIX, package=color(pkg.name, "YELLOW")
				)
			)
			return False
		depcache.mark_delete(pkg._pkg, arguments.is_purge())
		return True

	# Check the installed version against the candidate version in case we're downgrading or upgrading.
	if (
		pkg.installed
		and pkg.candidate
		and pkg.installed.version == pkg.candidate.version
	):
		print(
			_("{package} is already at the latest version {version}").format(
				package=color(pkg.name, "GREEN"),
				version=color(pkg.installed.version, "BLUE"),
			)
		)
		return False
	depcache.mark_install(pkg._pkg, False, True)
	return True


def sort_pkg_changes(pkgs: list[Package], nala_pkgs: PackageHandler) -> None:
	"""Sort a list of packages and splits them based on the action to take."""
	dprint("Sorting Package Changes")
	for pkg in pkgs:
		installed = get_pkg_version(pkg, inst_first=True)
		if pkg.marked_delete:
			delete, autoremove = (
				(nala_pkgs.delete_pkgs, nala_pkgs.autoremove_pkgs)
				if pkg.installed
				else (nala_pkgs.delete_config, nala_pkgs.autoremove_config)
			)
			npkg = NalaPackage(pkg.name, installed.version, installed.installed_size)
			if pkg.name in nala_pkgs.autoremoved:
				autoremove.append(npkg)
				continue
			delete.append(npkg)

		candidate = get_pkg_version(pkg, cand_first=True)
		npkg = NalaPackage(pkg.name, candidate.version, candidate.size)
		if pkg.marked_install:
			nala_pkgs.install_pkgs.append(npkg)
			continue

		if pkg.marked_reinstall:
			nala_pkgs.reinstall_pkgs.append(npkg)
			continue

		npkg = NalaPackage(
			pkg.name, candidate.version, candidate.size, installed.version
		)
		if pkg.marked_upgrade:
			nala_pkgs.upgrade_pkgs.append(npkg)
			continue

		if pkg.marked_downgrade:
			nala_pkgs.downgrade_pkgs.append(npkg)
			continue


def need_reboot() -> bool:
	"""Check if the system needs a reboot and notify the user."""
	if REBOOT_REQUIRED.exists():
		if REBOOT_PKGS.exists():
			print(
				_("{notice} The following packages require a reboot.").format(
					notice=NOTICE_PREFIX
				)
			)

			for pkg in REBOOT_PKGS.read_text(encoding="utf-8").splitlines():
				print(f"  {color(pkg, 'GREEN')}")
			return False
		return True
	return False


def print_notices(notices: Iterable[str]) -> None:
	"""Print notices from dpkg."""
	if notices:
		print("\n" + color(_("Notices:"), "YELLOW"))
		for notice_msg in notices:
			if "NOTICE:" in notice_msg:
				notice_msg = notice_msg.replace("NOTICE:", NOTICE_PREFIX)
			if "Warning:" in notice_msg:
				notice_msg = notice_msg.replace("Warning:", WARNING_PREFIX)
			print(f"  {notice_msg}")


def setup_cache() -> Cache:
	"""Update the cache if necessary, and then return the Cache."""
	try:
		if arguments.update:
			with DelayedKeyboardInterrupt():
				with DpkgLive(install=False) as live:
					Cache().update(UpdateProgress(live))
	except (LockFailedException, FetchFailedException, apt_pkg.Error) as err:
		apt_error(err, arguments.command == "update")
	except KeyboardInterrupt:
		eprint(_("Exiting due to SIGINT"))
		sys.exit(ExitCode.SIGINT)
	except BrokenPipeError:
		sys.stderr.close()
	try:
		cache = Cache(OpProgress())
	except apt_pkg.Error as err:
		apt_error(err, True)
	return cache


def sort_pkg_name(pkg: Package) -> str:
	"""Sort by package name.

	This is to be used as sorted(key=sort_pkg_name)
	"""
	return f"{pkg.name}"


def check_term_ask() -> None:
	"""Check terminal and ask user if they want to continue."""
	# If we're piped or something the user should specify --assume-yes
	# As They are aware it can be dangerous to continue
	if (
		not term.console.is_terminal
		and not arguments.assume_yes
		and not arguments.assume_no
	):
		sys.exit(
			_(
				"{error} It can be dangerous to continue without a terminal. Use `--assume-yes`"
			).format(error=ERROR_PREFIX)
		)

	if not arguments.fix_broken:
		print(
			_("{warning} Using {switch} can be very dangerous!").format(
				warning=WARNING_PREFIX, switch=color("--no-fix-broken", "YELLOW")
			)
		)

	if not ask(_("Do you want to continue?")):
		eprint(_("Abort."))
		sys.exit(0)


def check_work(pkgs: list[Package], nala_pkgs: PackageHandler, operation: str) -> None:
	"""Check if there is any work for nala to do.

	Returns None if there is work, exit's successful if not.
	"""
	if nala_pkgs.configure_pkgs:
		return
	if operation == "upgrade" and not pkgs:
		print(color(_("All packages are up to date.")))
		sys.exit(0)
	elif operation == "install" and not pkgs and not nala_pkgs.local_debs:
		print(color(_("Nothing for Nala to do.")))
		sys.exit(0)
	elif operation == "remove" and not pkgs:
		print(color(_("Nothing for Nala to remove.")))
		sys.exit(0)
	elif operation == "fix-broken" and not pkgs:
		print(color(_("Nothing for Nala to fix.")))
		sys.exit(0)


def check_essential(pkgs: list[Package]) -> None:
	"""Check removal of essential packages."""
	dprint(f"Checking Essential: {not arguments.remove_essential}")
	if arguments.remove_essential:
		return
	essential: list[Text] = []
	for pkg in pkgs:
		if pkg.is_installed:
			# do not allow the removal of essential or required packages
			if pkg.essential and pkg.marked_delete:
				essential.append(from_ansi(color(pkg.name, "RED")))
			# do not allow the removal of nala
			elif pkg.shortname in "nala" and pkg.marked_delete:
				essential.append(from_ansi(color("nala", "RED")))

	if essential:
		essential_error(essential)
