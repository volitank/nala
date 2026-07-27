# Nala

Nala is a command-line front-end for `libapt-pkg`, currently being rewritten in
Rust on top of [`rust-apt`](https://gitlab.com/volian/rust-apt).

The goal is the same as the original Python project: keep APT's package
semantics, but present package operations in a way that is easier to inspect
before committing changes. The Rust version keeps that focus while adding a
more structured command parser, typed configuration, JSON-friendly internal
models, and terminal UI paths for high-information workflows.

## Status

This repository is in transition from the original Python implementation to the
Rust implementation. The active application code lives under `src/`. Python-era
source, Poetry metadata, translation files, RST docs, and screenshots are
archived under [`legacy/python`](legacy/python).

## Features

- Package discovery commands: `list`, `search`, `show`, and `policy`
- Transaction commands: `install`, `remove`, `autoremove`, `update`, and
  `upgrade`
- Upgrade modes for normal, safe, and full upgrades
- Exclusion support for upgrades with glob-style package patterns
- Local `.deb` installation and direct URL package ingestion
- Parallel package downloader with per-domain connection limits
- Hash verification for package downloads before installation
- Proxy support through APT acquire configuration
- Transaction summaries in full, simple, plain, or TUI-backed views
- Transaction history stored as per-entry JSON records
- History inspection, `undo`, `redo`, and targeted or full history clearing
- Mirror selection through `nala fetch`, including automatic and interactive
  TUI modes
- Machine-readable output for package info and policy views
- HCL configuration for Nala behavior, UI mode, units, color mode, and theme
- Shell completion integration through `clap_complete`
- Generated manpage and Markdown command documentation from the Rust CLI

## Commands

Common command families:

```sh
nala update
nala upgrade
nala install ripgrep
nala remove ripgrep
nala autoremove
```

Package inspection:

```sh
nala list nala
nala search --names-only apt
nala show nala
nala policy nala
```

History:

```sh
nala history
nala history last
nala history undo last
nala history redo 12
nala history clear 12
nala history clear --all
```

Mirror selection:

```sh
nala fetch
nala fetch --auto
nala fetch --https-only --country US
```

Most transaction commands accept shared safety and behavior flags such as
`--download-only`, `--simple`, `--update`, `--no-update`, `--assume-yes`,
`--assume-no`, `--purge`, and `--allow-unauthenticated`.

## Configuration

Nala reads an HCL configuration file. See [`nala.conf`](nala.conf) for the
current shape.

Top-level configuration areas:

- `Nala`: transaction behavior such as auto remove, auto update, simple
  summaries, and assume-yes defaults
- `Ui`: output mode and binary or decimal unit formatting
- `Color`: color mode and theme entries for package summaries, progress, and
  warnings

Command-line flags override configuration values for the current invocation.
APT options can be passed through with `-o KEY=VALUE`.

## Development

Install system requirements first:

```sh
sudo apt-get install libapt-pkg-dev codespell
```

Build and test:

```sh
cargo build
cargo test -- --test-threads 1
```

Useful `just` targets:

```sh
just build
just test
just clippy
just fmt
just check
```

`just check` runs spellcheck, clippy, and nightly rustfmt checking. The
formatter target expects a nightly toolchain with `rustfmt` installed.

## Generated Files

The Rust build script generates manpages from the clap parser through
`clap_mangen`, keeping command documentation tied to the same parser that
powers the binary.

## Safety Notes

Package-changing commands require root privileges. Nala checks for root before
operations that modify the system, displays planned changes before committing
them, records applied transactions to history, and reports when the system marks
a reboot as required.

## Links

- Official repository: <https://gitlab.com/volian/nala>
- Issues: <https://gitlab.com/volian/nala/-/issues>
