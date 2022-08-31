#!/usr/bin/env just --justfile

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

# Run the tests
test +ARGS="":
	@cargo test -- --test-threads 1 {{ARGS}}

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
	@cargo clippy --all-targets --all-features --workspace -- --deny warnings {{ARGS}}
	@echo Lint successful!

# Format the codebase
fmt +ARGS="":
	@cargo +nightly fmt --all -- {{ARGS}}
	@echo Codebase formatted successfully!

# Spellcheck the codebase
spellcheck +ARGS="--skip target*":
	@codespell --skip="./po"--builtin clear,rare,informal,code --ignore-words-list mut,crate {{ARGS}}
	@echo Spellings look good!
