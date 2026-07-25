# Debian Package Fixtures

This directory holds Debian integration test package metadata.
Generated `.deb` files are built in a temp directory by `run`.

Current package:

- `broken-or-deps.control`
- version `1.0`
- architecture `all`
- depends on `missing-a (>= 1.0) | missing-b (>= 1.0)`

Neither dependency is available. `apt-get install -s ./broken-or-deps.deb`
and `nala install ./broken-or-deps.deb` should both fail with unmet dependencies.
