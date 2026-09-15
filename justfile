#!/usr/bin/env just --justfile

[private]
default:
    @just --list

# Setup the development environment
setup-dev:
    @echo Installing required packages from apt
    @sudo apt-get install g++ libapt-pkg-dev pkgconf codespell -y

    @echo Setting up toolchains
    @rustup toolchain install nightly
    @rustup toolchain install stable

    @echo Installing nightly \`rustfmt\`
    @rustup toolchain install nightly --component rustfmt
    @echo Nightly \`rustfmt\` successfully installed!

    @echo Development environment installed successfully!

# Run checks
check: spellcheck clippy
    @cargo +nightly fmt --check
    @echo Checks were successful!

# Remove generated artifacts
clean:
    @cargo clean
    @echo Done!

# Build the project
build:
    @cargo build
    @echo Project successfully built!

# Build with custom release profile
release:
    @cargo build --profile=lto

# Build release and install the binary
install: release
    sudo cp target/lto/nala /usr/bin/nala

# Build the Debian package while keeping build caches
deb:
    scripts/deb-build --binary --no-clean

# Build the same Debian package with locked crates.io dependencies
deb-upstream:
    scripts/deb-build --binary --no-clean --upstream-cargo

# Build source Debian artifacts
deb-source:
    scripts/deb-build --source

# Build source and binary Debian artifacts
deb-release:
    scripts/deb-build --release

# Build signed source and binary Debian artifacts
deb-release-sign KEY:
    scripts/deb-build --release --key-id {{ KEY }}

# Start a Debian Sid shell with the strict package build dependencies installed
sid:
    @docker build --quiet -f tests/deb/Dockerfile -t nala-sid tests/deb
    @docker run --rm -it \
        -e TERM="${TERM:-dumb}" \
        -v "$PWD:/work" \
        -w /work \
        nala-sid \
        bash -lc 'apt-get update && apt-get build-dep -y --no-install-recommends . && exec bash'

# Run a command in the Debian Sid test container
[private]
sid-run COMMAND:
    @docker build --quiet -f tests/deb/Dockerfile -t nala-sid tests/deb
    @docker run --rm \
        -e CARGO_TARGET_DIR=/target \
        -e TERM="${TERM:-dumb}" \
        -v "$PWD:/work" \
        -v nala-sid-target:/target \
        -w /work \
        nala-sid \
        sh -c '{{ COMMAND }}'

# Run Debian package integration test
debtest:
    @just sid-run 'cargo build --locked --quiet && tests/deb/run && tests/deb/history/run && tests/deb/safety/run'

# Run history package integration test
history-test:
    @just sid-run 'cargo build --locked --quiet && tests/deb/history/run'

# Run transaction safety integration tests
safety-test:
    @just sid-run 'cargo build --locked --quiet && tests/deb/safety/run'

# Run terminal integration test
term-test:
    @just sid-run 'cargo build --locked --quiet && tests/deb/term/run'

# Print deterministic previews of the TUI screens
tui-preview:
    @just sid-run 'cargo test --locked --quiet tui_preview -- --ignored --nocapture --test-threads 1'

# Run the tests
test +ARGS="":
    @cargo test -- --test-threads 1 {{ ARGS }}

# Run leak tests. Requires root
leak:
    @cargo test --no-run
    @sudo valgrind --leak-check=full -- $( \
        find target/debug/deps/ \
        -executable \
        -type f \
        -name "tests-*" \
        -printf "%T@ %p\n" | sort -nr | awk '{print $2}' \
    ) --test-threads 1

# Lint the codebase
clippy +ARGS="":
    @cargo clippy --all-targets --all-features --workspace -- --deny warnings {{ ARGS }}
    @echo Lint successful!

# Format the codebase
fmt +ARGS="":
    @cargo +nightly fmt --all -- {{ ARGS }}
    @echo Codebase formatted successfully!

# Spellcheck the active codebase
spellcheck +ARGS="":
    @codespell --skip="./target*,./legacy/*,./locales/*,./.cargo/*,./debian/generated-man/*" --builtin clear,rare,informal,code --ignore-words-list mut,crate,ratatui,stdio,ws {{ ARGS }}
    @echo Spellings look good!
