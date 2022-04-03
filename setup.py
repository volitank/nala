#!/usr/bin/python3
"""Setup module."""
from pathlib import Path

import tomli
from DistUtilsExtra.auto import clean_build_tree
from DistUtilsExtra.command import build_extra, build_i18n
from setuptools import setup

PYPROJECT = Path("./pyproject.toml")
POETRY = tomli.loads(PYPROJECT.read_text(encoding="utf-8"))["tool"]["poetry"]

setup(
	name=POETRY["name"],
	description=POETRY["description"],
	version=POETRY["version"],
	author=POETRY["authors"][0].split()[0],
	author_email=POETRY["authors"][0].split()[1],
	cmdclass={
		"build": build_extra.build_extra,
		"build_i18n": build_i18n.build_i18n,
		"clean": clean_build_tree,
	},
	license=POETRY["license"],
	packages=["nala"],
	entry_points={
		"console_scripts": [
			"nala=nala.__main__:main",
		],
	},
)
