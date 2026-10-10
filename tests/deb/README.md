# Debian integration tests

These tests run the built Nala binary against real APT and dpkg state in a
disposable Debian Sid environment. Package fixtures use ordinary `DEBIAN/`
directories under `packages/` and are built into temporary `.deb` files.

- `run` checks text/JSON package and global policy against an installed-only
  status database, then checks that APT and Nala reject unsatisfiable dependency
  alternatives and select a valid fallback recommendation.
- `history/run` exercises conversion, package lifecycle, replay, clearing, and
  failure handling against an offline APT repository and real dpkg state.
- `term/run` installs the whiptail fixture in tmux, resizes the terminal while
  the dialog is open, and verifies that output resumes at column zero.

Run `just debtest`, `just history-test`, or `just term-test` from the
repository root. GitLab CI runs the same scripts against the packaged binary:
unoptimized on branches and optimized with LTO on tags.

CI also reruns unit tests with a German locale and `tests/empty-apt.conf`,
which hides the builder's installed packages and APT indexes. Cache tests
must supply their own fixtures; output assertions must use translated labels.
Debian package builds run tests serially because APT and logging have global
state. To run the same check after a strict Debian build, reconfigure first
because `scripts/deb-build` restores the upstream lockfile:

```sh
dh_auto_configure --buildsystem cargo
LC_ALL=de_DE.UTF-8 APT_CONFIG="$PWD/tests/empty-apt.conf" \
  dh_auto_test --buildsystem cargo -- test -- --test-threads 1
```

Run `just sid` for an interactive Sid shell with Nala's strict Debian build
dependencies installed. Its `target` directory belongs to the source bind
mount, so `just deb-release` can perform a clean package build.
