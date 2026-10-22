# Releasing Nala

Update `Cargo.toml`, Nala's `Cargo.lock` entry, and `debian/changelog` together.
Add a `debian/NEWS` entry only for user-facing changes that need an upgrade
notice, such as configuration migration. The Cargo version and Debian version
must match the release tag, for example `0.18.0` and `v0.18.0`.

After committing and pushing, let the branch pipeline pass before pushing the
release tag. CI runs the same four package jobs for tags: Debian Sid and the
upstream-Cargo package on Ubuntu 24.04, both on amd64 and arm64.

Branch jobs use unoptimized builds with debug assertions and overflow checks.
Tag jobs use size optimization and LTO; prerelease tags also use these settings
but do not publish a release.

Each Sid job builds an unsigned source package, extracts its `.dsc`, rebuilds
the binary package, and runs Lintian and the Debian integration suites. Ubuntu
24.04 uses its own libapt-pkg and locked upstream Cargo dependencies, runs the
package integration smoke, and installs the resulting package.

## GitLab releases and downloads

Only tags matching `vMAJOR.MINOR.PATCH` publish downloads and create a release.
Branch builds and prerelease tags retain ordinary one-week CI artifacts.
`scripts/ci-release-assets` checks the tag, Cargo and Debian versions, and
binary package version/architecture before uploading anything.

After its checks pass, each build job uploads its `.deb` and complete build
bundle to the project's Generic Package Registry using `CI_JOB_TOKEN`. The
release job runs after all build jobs succeed and creates eight download links:

- Debian Sid amd64 and arm64 `.deb` files and complete source/build bundles.
- Ubuntu 24.04 amd64 and arm64 `.deb` files and complete binary build bundles.

Bundles contain `dist/debian/` and build logs. Sid bundles also include the
`.dsc`, source archive, source and binary `.changes`/`.buildinfo`, and debug
package. Release notes come from the newest Debian changelog entry.

These downloads remain in the package registry after CI artifacts expire;
keep released package versions out of any registry cleanup policy. Package
Registry access must allow public downloads. No additional CI secret is
needed. Create/push the tag without manually creating a GitLab release first:
GitLab's `release:` job fails if a release already exists for that tag.

Use the Ubuntu build on Ubuntu 24.04; the Sid build requires libapt-pkg7.0 and
must not replace legacy packages in a shared repository for older systems.
Other distributions and architectures require their own build and
installation validation.

## Debian archive upload

GitLab downloads are unsigned. Download and unpack one Sid bundle, review its
source/build logs and Lintian results, then sign the source `.changes` with
`debsign` using your Debian upload key; it also signs the referenced `.dsc`
and `.buildinfo`. Upload that signed source with the usual Debian tooling.
Do not mix the two architecture bundles or upload the Ubuntu package to Sid.
The GitLab release job does not sign or upload anything to Debian or Volian.
