#!/usr/bin/env just --justfile

[private]
default:
    @just --list

# Setup the development environment
setup-dev:
    @echo Installing required packages from apt
    @sudo apt-get install libapt-pkg-dev codespell -y

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

# Build source Debian artifacts
deb-source:
    scripts/deb-build --source

# Build source and binary Debian artifacts
deb-release:
    scripts/deb-build --release

# Build signed source and binary Debian artifacts
deb-release-sign KEY:
    scripts/deb-build --release --key-id {{ KEY }}

# Run a command in the Debian test container
docker COMMAND:
    @docker build --quiet -f tests/deb/Dockerfile -t nala-debtest tests/deb
    @docker run --rm \
        -e CARGO_TARGET_DIR=/target \
        -e TERM="${TERM:-dumb}" \
        -v "$PWD:/work" \
        -v nala-debtest-target:/target \
        -w /work \
        nala-debtest \
        sh -c '{{ COMMAND }}'

# Run Debian package integration test
debtest:
    @just docker 'cargo build --locked --quiet && tests/deb/run'

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
    @codespell --skip="./target*,./legacy/*,./locales/*,./.cargo/*" --builtin clear,rare,informal,code --ignore-words-list mut,crate,ratatui,stdio,ws {{ ARGS }}
    @echo Spellings look good!
