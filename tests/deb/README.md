# Debian integration tests

These tests run the built Nala binary against real APT and dpkg state in a
disposable Debian Sid environment. Package fixtures use ordinary `DEBIAN/`
directories under `packages/` and are built into temporary `.deb` files.

- `run` checks that APT and Nala reject unsatisfiable dependency alternatives
  and select a valid fallback recommendation.
- `history/run` exercises conversion, package lifecycle, replay, clearing, and
  failure handling against an offline APT repository and real dpkg state.
- `term/run` installs the whiptail fixture in tmux, resizes the terminal while
  the dialog is open, and verifies that output resumes at column zero.

Run `just debtest`, `just history-test`, or `just term-test` from the
repository root. GitLab CI runs the same scripts against the release binary.

Run `just sid` for an interactive Sid shell with Nala's strict Debian build
dependencies installed. Its `target` directory belongs to the source bind
mount, so `just deb-release` can perform a clean package build.
