#!/usr/bin/python3
"""Management tools related to building Nala."""
import os
import sys
import time
from pathlib import Path
from subprocess import run

import typer

from nala import USR, __version__ as version

PO_FILES = tuple(Path("po").glob("*.po"))
SOURCE_FILES = tuple(Path("nala").glob("*.py"))
DOCS_DIR = Path("docs")

# pylint: disable=too-few-public-methods
class BuildEnvironment:
	"""Hold environment variables."""

	def __init__(self, build_dir: str) -> None:
		"""Hold environment variables."""
		self.build_dir = build_dir
		self.bin_dir = f"{build_dir}{USR}/bin"
		self.locale_dir = f"{build_dir}{USR}/share/locale"


nala_app = typer.Typer(add_completion=False, no_args_is_help=True)


def check_root(operation: str) -> None:
	"""Check for root and exit if not."""
	if os.getuid() != 0:
		sys.exit(f"Error: You need root to install the {operation}.")


@nala_app.command(name="man")
def convert_man(
	install: bool = typer.Option(
		False, "--install", help="Additionally install the man pages"
	)
) -> None:
	"""Convert .rst files into man pages."""
	if install:
		check_root("man pages")

	date = time.strftime(
		"%d %b %Y",
		time.gmtime(int(os.environ.get("SOURCE_DATE_EPOCH", time.time()))),
	)

	# Convert man page and install if requested
	for file in DOCS_DIR.iterdir():
		if not file.name.endswith(".rst"):
			continue

		# If the install switch is set then we install it to the man directory.
		man_page = Path(
			f"{USR}/share/man/man8/{file.name.replace('.rst', '')}"
			if install
			else f"{file}".replace(".rst", "")
		)

		print(f"Installing {file} -> {man_page}")

		pandoc = [
			"pandoc",
			f"{file}",
			f"--output={man_page}",
			"--standalone",
			"--variable=header:'Nala User Manual'",
			f"--variable=footer:{version}",
			f"--variable=date:{date}",
			"--variable=section:8",
			"--from",
			"rst",
			"--to",
			"man",
		]
		run(pandoc, check=True)


@nala_app.command(name="nuitka")
def run_nuitka(
	_compile: bool = typer.Option(
		False, "--compile", help="Compile '.po' files to '.mo'"
	)
) -> None:
	"""Compile Nala with Nuitka."""
	env = BuildEnvironment(build_dir="debian/nala-legacy")
	if _compile:
		compile_translations(env)

	follow_imports = (
		"nala",
		"rich",
		"anyio",
		"typer",
		"click",
		"httpx",
		"httpcore",
		"h11",
		"charset_normalizer",
		"socksio",
		"typing_extensions",
		"pexpect",
		"ptyprocess",
		"pygments",
		"rfc3986",
		"sniffio",
		"certifi",
		"idna",
		"tomli",
	)
	nuitka = [
		"nuitka3",
		"--assume-yes-for-downloads",
		"--plugin-enable=pylint-warnings",
		"--remove-output",
		"nala-cli.py",
		"-o",
		f"{env.bin_dir}/nala",
	]

	nuitka.extend(f"--include-package={mod}" for mod in follow_imports)
	run(nuitka, check=True)
	run(f"chrpath -d {env.bin_dir}/nala".split(), check=True)


def update_translations() -> None:
	"""Update the .po files from the pot file."""
	update = "pybabel update --no-wrap -i po/nala.pot".split()
	for path in PO_FILES:
		# Strip off `.po`
		run(update + ["-o", f"po/{path.name}", "-l", path.name[:-3]], check=True)


def compile_translations(env: BuildEnvironment) -> None:
	"""Compile .po files to .mo."""
	pybable = f"pybabel compile --directory={env.locale_dir} --domain=nala --use-fuzzy".split()
	for path in PO_FILES:
		# Strip off `.po`
		locale = path.name[:-3]
		Path(f"{env.locale_dir}/{locale}/LC_MESSAGES/").mkdir(
			parents=True, exist_ok=True
		)

		compile_mo = pybable + [f"--input-file=po/{locale}.po", f"--locale={locale}"]
		run(compile_mo, check=True)


def extract_translations() -> None:
	"""Extract translations to nala.pot."""
	run(
		[
			"pybabel",
			"extract",
			"--add-comments=NOTE:",
			"--strip-comments",
			"--project=nala",
			"--copyright-holder=Blake Lee",
			f"--version={version}",
			"--msgid-bugs-address=https://gitlab.com/volian/nala/-/issues",
			"--no-wrap",
			*SOURCE_FILES,
			"-o",
			"po/nala.pot",
		],
		check=True,
	)


@nala_app.command()
def babel(
	extract: bool = typer.Option(
		False,
		"--extract",
		help="Extract translations to nala.pot and update the po files.",
	),
	_compile: bool = typer.Option(False, "--compile", help="Compile .po files to .mo"),
	install: bool = typer.Option(
		False, "--install", help="Additionally install the translation files."
	),
) -> None:
	"""Manage translation files."""
	if extract:
		extract_translations()
		update_translations()
	elif _compile:
		if install:
			check_root("translation files")

		compile_translations(
			BuildEnvironment(build_dir="")
			if install
			else BuildEnvironment(build_dir="debian/nala"),
		)
	else:
		sys.exit("Error: You need to specify either '--compile' or '--extract'")


if __name__ == "__main__":
	nala_app()
