# Legacy Python Nala

This directory archives the original Python implementation while the active
project moves to the Rust code under `../../src`.

The files here are kept together so old source, build scripts, Poetry metadata,
translations, documentation, and screenshots remain available for reference.
They are not part of the active Rust build.

Archived entry points:

- `nala/`: original Python package
- `pyproject.toml` and `poetry.lock`: Python package metadata and lockfile
- `nala_build.py`: old manpage and translation helper
- `nala-pyinstall.sh`: old PyInstaller helper
- `Makefile`: old Python install and cleanup targets
- `po/`: translation catalog files from the Python implementation
- `docs/`: RST/manual-page sources from the Python implementation
- `imgs/`: screenshots used by the old README
Most archived commands assume they are run from this directory.
