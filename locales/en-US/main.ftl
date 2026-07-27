### English messages displayed by Nala.

## General labels and errors

unknown = Unknown
none = None
error = Error
mirrors = Mirrors:

log-error = Error:
log-notice = Notice:
log-warning = Warning:
log-info = Info:
log-verbose = Verbose:
log-debug = Debug:

not-implemented = Not Yet Implemented.
subcommand-missing = Subcommand not supplied

# Variables:
#   $response (String) - Invalid confirmation response.
prompt-invalid = '{ $response }' is not a valid response
prompt-refused = User refused confirmation
prompt-continue = Do you want to continue?
prompt-choice = [Y/n]

# Variables:
#   $command (String) - Command that requires elevated privileges.
root-required = Nala needs root to { $command }

## Transaction operations and reasons

op-remove = Remove
op-auto-remove = AutoRemove
op-purge = Purge
op-auto-purge = AutoPurge
op-install = Install
op-reinstall = ReInstall
op-upgrade = Upgrade
op-downgrade = Downgrade
op-configure = Configure
op-held = Held

reason-excluded = Excluded
reason-manual-hold = Manual hold
reason-phased = Phased
reason-kept-back = Kept back

# Variables:
#   $percentage (Number) - Phased update percentage.
reason-phased-percent = Phased { $percentage }%

## Transaction summary

summary-title = Summary
summary-total-download = Total download size:
summary-disk-required = Disk space required:
summary-disk-free = Disk space to free:

summary-total-download-value = Total download size: { $size }
summary-disk-required-value = Disk space required: { $size }
summary-disk-free-value = Disk space to free: { $size }
summary-op-count = { $operation } { $count }
summary-op-count-colon = { $operation }: { $count }

summary-nothing = Nothing to do.
summary-essential = The following packages are essential!
summary-remove-essential = You have attempted to remove essential packages

# Variables:
#   $switch (String) - Command-line switch that permits essential package removal.
summary-use-switch = Use '{ $switch }' if you are sure.

summary-reboot = A reboot is required to complete these changes.
summary-reboot-packages = The following packages require a reboot:

## Package summary table

table-package = Package:
table-version = Version:
table-old-version = Old Version:
table-new-version = New Version:
table-reason = Reason:
table-size = Size:

## Package selection

# Variables:
#   $package (String) - Package name or selection token.
pkg-not-found = '{ $package }' was not found
pkg-not-found-cache = Some packages were not found in the cache
pkg-no-candidate = { $package } has no install candidate
pkg-mark-install = Unable to mark '{ $package }' for installation
pkg-not-installed = { $package } is not installed
pkg-reinstall-missing = { $package } is not installed, so it cannot be reinstalled
pkg-invalid-name = Invalid package name: '{ $package }'

# Variables:
#   $package (String) - Package name.
#   $operation-a (String) - First requested operation.
#   $operation-b (String) - Conflicting requested operation.
pkg-op-conflict = Conflicting operations for '{ $package }': { $operation-a } vs { $operation-b }

# Variables:
#   $package (String) - Package name.
#   $version-a (String) - First requested version.
#   $version-b (String) - Conflicting requested version.
pkg-version-conflict = Conflicting pinned versions for '{ $package }': { $version-a } vs { $version-b }

# Variables:
#   $package (String) - Package name.
#   $version (String) - Package version.
pkg-already-latest = { $package }{ $version } is already installed and at the latest version
pkg-version-missing = Unable to find version '{ $version }' for '{ $package }'
pkg-cache-missing = Package '{ $package }' not found in cache
pkg-recorded-version-missing = No recorded version for '{ $package }'
pkg-version-cache-missing = Version '{ $version }' not found for '{ $package }'

# Variables:
#   $pin (String) - Version pin supplied by the user.
pkg-invalid-pin = Invalid version pin: '{ $pin }'
pkg-pin-exact = Version pin requires an exact package name: '{ $pin }'

# Variables:
#   $pattern (String) - Package exclusion pattern.
pkg-exclude-missing = Exclude pattern '{ $pattern }' did not match any packages

# Variables:
#   $package (String) - Virtual package name.
pkg-virtual-no-providers = { $package } has no providers and is purely virtual
pkg-virtual-providers = { $package } is a virtual package provided by:

# Variables:
#   $package (String) - Virtual package name.
#   $provider (String) - Selected concrete provider.
pkg-virtual-select = Selecting { $provider } instead of virtual package { $package }
pkg-virtual-one = You should select just one.

# Variables:
#   $selected (String) - Selected package name.
#   $requested (String) - Originally requested package name.
pkg-select = Selecting Package '{ $selected }' instead of '{ $requested }'

pkg-fix-broken = Unable to correct broken packages
pkg-required = You must specify a package

## Download output

download-complete = Downloads Complete:
download-failed = Downloads Failed
download-some-missing = Some packages were not found.
download-auth-warning = The following packages cannot be authenticated!
download-auth-allowed = Configuration is set to allow installation of unauthenticated packages.
download-auth-required =
    Some packages were unable to be authenticated.
      If you're sure use { $switch }
download-exit = Exiting at user request
download-items = Items

# Variables:
#   $package (String) - Package name.
#   $version (String) - Package version.
download-source-missing = Can't find a source to download version '{ $version }' of '{ $package }'

# Variables:
#   $package (String) - Package name.
#   $path (String) - Destination path.
download-written = { $package } was written to { $path }

# Variables:
#   $uri (String) - Download URI.
download-hash-missing = No Hash Found for '{ $uri }'
download-protocol-missing = No protocol was defined
download-uri-missing = No uri was defined
download-malformed = '{ $uri }' is malformed!
download-request-failed = Download request failed for '{ $uri }'
download-stream-failed = Unable to stream data from '{ $uri }'
download-get = Unable to send download request
download-content-length = content-length does not exist in { $headers }
download-content-str = Converting content-length to a string failed: { $headers }
download-content-parse = Parsing content-length failed: { $headers }

# Variables:
#   $file (String) - Downloaded filename.
download-checksum = Checksum did not match for { $file }
download-no-uris = No URIs could be downloaded for { $file }

# Variables:
#   $path (String) - Path without a valid filename.
download-filename = { $path } does not have a valid filename!

download-error = Error: { $error }
download-select-domain = Selecting { $domain } for { $file }
download-start = Starting: { $uri }, Retries: { $retries }
download-finished = Finished: { $uri }

## Progress

progress-working = Working...
progress-nothing = Nothing to fetch
progress-label = Progress
progress-total = Total
progress-speed = Speed
progress-elapsed = Elapsed
progress-remaining = Remaining
progress-finished = Finished:

# Variables:
#   $size (String) - Total amount fetched.
#   $time (String) - Elapsed time.
#   $rate (String) - Transfer rate.
progress-fetched = Fetched { $size } in { $time } ({ $rate }/s)

## TUI

tui-upgrade-title = Nala Upgrade
tui-history-title = Nala History
tui-pkgs = Pkgs
tui-help-move = (↑) move up | (↓) move down
tui-help-tabs = (→) next tab | (←) previous tab
tui-help-show = (Enter) show changelog | (s) show version info
tui-help-confirm = (q) quit | (y) start upgrade
tui-help-quit = (q) quit
tui-changelog-missing = Unable to find Changelog URI

## Files, configuration, and package archives

# Variables:
#   $path (String) - File or directory path.
file-remove = Failed to remove { $path }
file-read = Failed to read '{ $path }'
file-read-defaults = Failed to read { $path }, using defaults
file-parse-defaults = Failed to parse { $path }, using defaults
file-create = Unable to create '{ $path }'
file-write = Unable to write to '{ $path }'
file-replace = Unable to replace '{ $path }'
file-deserialize = Unable to deserialize '{ $path }'

# Variables:
#   $operation (String) - Filesystem operation.
#   $path (String) - Source path.
fs-failed = Failed to { $operation } { $path }

# Variables:
#   $operation (String) - Filesystem operation.
#   $path (String) - Source path.
#   $target (String) - Destination path.
fs-failed-target = Failed to { $operation } { $path } => { $target }

# Variables:
#   $option (String) - Unsupported APT configuration override.
config-option = Option '{ $option }' is not supported
color-expected = a color name, 0-255 index, or #RRGGBB hex string
color-index = color index must be between 0 and 255
color-unknown = unknown color '{ $color }'
color-rgb-components = Rgb expects three components
color-rgb-expected = expected Rgb or Indexed color
color-modifier-expected = a modifier string like "BOLD | ITALIC" or an array
color-modifier-unknown = unknown modifier '{ $modifier }'

archive-unsupported = Archive type is not supported

# Variables:
#   $path (String) - Debian archive path.
deb-control-missing = control file not found in { $path }

# Variables:
#   $type (String) - Hash type.
hash-unsupported = Hash Type: { $type } is not supported

# Variables:
#   $package (String) - Package name.
#   $version (String) - Package version.
hash-unavailable =
    { $package } { $version } can't be checked for integrity.
    There are no hashes available for this package.

## Dpkg

dpkg-child-failed = Dpkg child failed: { $error }
dpkg-status-utf8 = Dpkg status fd returned invalid UTF-8
dpkg-exit = Dpkg exited with code: '{ $code }'
dpkg-poll = Unable to poll child
dpkg-read-status = Unable to read Status Fd
dpkg-read-pty = Unable to read from pty
dpkg-write-pty = Unable to send stdin to pty
dpkg-removing = Removing:
dpkg-unpacking = Unpacking:
dpkg-setting-up = Setting up:
dpkg-processing = Processing:

## List

list-virtual = Virtual

## Show: general output

show-local-install = local install

# Variables:
#   $count (Number) - Number of additional package records.
#   $switch (String) - Command-line switch that displays every record.
show-more-records = There are { $count } additional records. Please use the { $switch } switch to see them.

show-no-description = No Description
show-no-summary = No Summary

## Show: package record labels

show-package = Package
show-version = Version
show-architecture = Architecture
show-priority = Priority
show-essential = Essential
show-section = Section
show-source = Source
show-installed-size = Installed-Size
show-size = Size
show-maintainer = Maintainer
show-original-maintainer = Original-Maintainer
show-homepage = Homepage
show-sha256 = SHA256
show-archive = Archive
show-origin = Origin
show-codename = Codename
show-component = Component
show-provides = Provides
show-description = Description
show-attributes = Attributes
show-apt-sources = APT-Sources
show-depends = Depends
show-pre-depends = PreDepends
show-suggests = Suggests
show-recommends = Recommends
show-conflicts = Conflicts
show-replaces = Replaces
show-obsoletes = Obsoletes
show-breaks = Breaks
show-enhances = Enhances

## Show: package attributes

show-attr-installed = Installed
show-attr-local = Local
show-attr-auto-removable = Auto-Removable
show-attr-automatic = Automatic

# Variables:
#   $version (String) - Package version available for upgrade.
show-attr-upgradable-to = Upgradable to: { $version }

# Variables:
#   $version (String) - Currently installed package version.
show-attr-upgradable-from = Upgradable from: { $version }

## Policy

policy-none = none
policy-installed = Installed:
policy-candidate = Candidate:
policy-version-table = Version table:
policy-no-versions = No versions.
policy-origin = origin
policy-package-files = Package files:
policy-pinned = Pinned packages:
policy-release = release

## History: general output

# Variables:
#   $count (Number) - Number of history entries that were cleared.
history-cleared =
    { $count ->
        [one] Cleared { $count } history entry.
       *[other] Cleared { $count } history entries.
    }

history-empty = No history entries found.

# Variables:
#   $id (Number) - ID of the history entry.
history-not-found = History entry with ID '{ $id }' does not exist
history-not-replayable = History entry '{ $id }' is not replayable because it was not recorded as applied
history-no-changes = History entry '{ $id }' has no package changes to replay
history-cleared-entry = Cleared history entry { $id }.

## History: labels

history-id = ID
history-command = Command
history-date-time = Date and Time
history-requested-by = Requested-By
history-altered = Altered
history-status = Status
history-status-applied = Applied
history-started = Started
history-finished = Finished
history-targets = Requested Targets

## History: replay errors

# Variables:
#   $package (String) - Package name.
history-undo-version-missing = Undo is not supported for '{ $package }' because the prior version was not recorded
history-undo-config-only = Undo is not supported for '{ $package }' because restoring config-files-only state is not implemented
history-undo-installed-version-missing = Undo is not supported for '{ $package }' because the prior installed version was not recorded
history-undo-reinstall = Undo is not supported for '{ $package }' because reinstall has no recorded inverse
history-undo-held = Held package '{ $package }' cannot be undone
history-redo-version-missing = Redo is not supported for '{ $package }' because the resulting version was not recorded
history-redo-reinstall-missing = Redo is not supported for '{ $package }' because the reinstalled version was not recorded
history-redo-held = Held package '{ $package }' cannot be redone

## History: selector and storage errors

history-selector = Invalid history selector '{ $value }'. Use an integer ID or 'last'.
history-clear-target = History clear requires an entry selector or --all
history-config-undo = Configured package '{ $package }' cannot be undone
history-config-redo = Configured package '{ $package }' cannot be redone
history-serialize = Unable to serialize history entry

## Fetch

fetch-no-mirrors = Nala was unable to find any mirrors.
fetch-none-selected = No mirrors were selected.
fetch-release-detect = There was an issue detecting release.

# Variables:
#   $distro (String) - Distribution name.
fetch-unsupported = { $distro } is unsupported.

# Variables:
#   $file (String) - Sources file path.
fetch-sources-written = Sources have been written to { $file }

fetch-title = Nala Fetch
fetch-score = Score:
fetch-score-help = Score is how many milliseconds it takes to download the Release file.
fetch-help = Use ↓↑ to move, Space to select/unselect, Home/End to go top/bottom, q/Enter to exit.

## Update

# Variables:
#   $count (Number) - Number of upgradable packages.
#   $command (String) - Command that lists upgradable packages.
update-upgradable =
    { $count ->
        [one] { $count } package can be upgraded. Run '{ $command }' to see it.
       *[other] { $count } packages can be upgraded. Run '{ $command }' to see them.
    }

update-no-change = No Change
update-updated = Updated
update-ignored = Ignored
update-downloading = Downloading
update-processing = Processing
update-item = { $state }: { $description }

## Upgrade

upgrade-exclude-unsafe =
    Selected packages cannot be excluded from upgrade safely.
    { $error }
upgrade-protect = Protecting { $package } from { $reason }
upgrade-reason-upgrade = upgrade
upgrade-reason-auto-remove = auto-removal
upgrade-config-missing = No config tree!

## Install

install-downloaded = Downloaded: { $path }

## CLI

cli-about = Commandline front-end for libapt-pkg
cli-license = Print license information
cli-verbose = Disable scrolling text and print extra information
cli-debug = Print debug statements for solving issues
cli-config = Specify a different configuration file
cli-tui = Turn on the TUI if it is disabled in the config
cli-no-tui = Turn the TUI off; takes precedence over other options
cli-option = Passthrough APT configurations
cli-color = Set color mode (always, never, auto)

cli-list = List all packages or only packages matching the provided name
cli-search = Search package names and descriptions using regular expressions
cli-show = Show information about one or more packages
cli-policy = Show pin and priority information about one or more packages
cli-clean = Remove locally downloaded package files
cli-download = Download packages to the current directory
cli-history = View or replay stored package transaction history
cli-fetch = Fetch fast mirrors for the current distribution
cli-update = Update package lists
cli-upgrade = Upgrade packages
cli-install = Install packages
cli-remove = Remove packages
cli-autoremove = Automatically remove unnecessary packages

cli-pkg-search = Package names to search
cli-pkg-show = Package names to show
cli-pkg-policy = Package names to show policy for
cli-pkg-download = Package names to download
cli-pkg-install = Package names to install
cli-pkg-remove = Package names to remove
cli-description = Print the full description of each package
cli-summary = Print the summary of each package
cli-installed = Only include installed packages
cli-nala-installed = Only include packages explicitly installed with Nala
cli-upgradable = Only include upgradable packages
cli-virtual = Only include virtual packages
cli-machine = Print machine-readable output
cli-names-only = Search only package names, not descriptions
cli-all-versions = Show all versions of a package
cli-all-arches = Show packages for all configured architectures
cli-clean-lists = Remove package lists downloaded by update
cli-clean-fetch = Remove the sources file generated by fetch
cli-history-id = Show details for a specific history entry ID or 'last'
cli-history-action = Run a history action instead of showing list or detail output
cli-history-undo = Replay the inverse of an applied history entry
cli-history-redo = Replay an applied history entry again
cli-history-clear = Clear stored history entries
cli-history-transaction-id = History entry ID or 'last'
cli-history-clear-id = History entry ID or 'last' to clear
cli-history-clear-all = Clear all stored history entries
cli-fetch-non-free = Include contrib and non-free repository components
cli-fetch-https = Only use HTTPS mirrors
cli-fetch-sources = Include source package repositories
cli-fetch-auto = Automatically choose the specified number of mirrors
cli-fetch-country = Restrict mirrors to a country code
cli-fetch-debian = Override the Debian release
cli-fetch-ubuntu = Override the Ubuntu release
cli-fetch-devuan = Override the Devuan release
cli-print-uris = Print URIs as JSON without upgrading
cli-exclude = Exclude packages from upgrade; accepts glob patterns
cli-full = Perform a full upgrade
cli-no-full = Do not perform a full upgrade
cli-safe = Perform a safe upgrade; takes precedence over other upgrade options
cli-reinstall = Reinstall packages that are already installed
cli-target-release = Set the default release to install packages from
cli-download-only = Only download packages
cli-simple = Display a simpler, condensed transaction summary
cli-update-first = Update package lists before running the command
cli-no-update = Do not update package lists before running the command
cli-allow-unauthenticated = Allow packages that cannot be authenticated
cli-assume-yes = Assume yes for all prompts
cli-assume-no = Assume no for all prompts
cli-remove-essential = Allow removal of essential packages
cli-purge = Remove configuration files for packages being removed
cli-fix-broken = Try to fix broken packages
cli-no-fix-broken = Do not try to fix broken packages
cli-install-recommends = Install recommended packages
cli-no-install-recommends = Do not install recommended packages
cli-install-suggests = Install suggested packages
cli-no-install-suggests = Do not install suggested packages
cli-auto-remove = Also remove unnecessary packages
cli-no-auto-remove = Do not remove unnecessary packages
cli-remove-config = When purging, also remove packages in config-files-only state
