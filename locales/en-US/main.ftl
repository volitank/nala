### English messages displayed by Nala.

## History: general output

# Variables:
#   $count (Number) - Number of history entries that were cleared.
history-cleared =
    { $count ->
        [one] Cleared { $count } history entry.
       *[other] Cleared { $count } history entries.
    }

# Variables:
#   $id (Number) - ID of the cleared history entry.
history-cleared-entry = Cleared history entry { $id }.

history-empty = No history entries found.

# Variables:
#   $id (Number) - ID of the requested history entry.
history-not-found = History entry with ID '{ $id }' does not exist

# Variables:
#   $id (Number) - ID of the history entry that cannot be replayed.
history-not-replayable = History entry '{ $id }' is not replayable because it was not recorded as applied

# Variables:
#   $id (Number) - ID of the history entry without package changes.
history-no-changes = History entry '{ $id }' has no package changes to replay

## History: replay errors

# Variables:
#   $version (String) - Requested package version.
#   $package (String) - Package name.
history-version-missing = Version '{ $version }' not found for '{ $package }'

# Variables:
#   $package (String) - Package name.
history-reinstall-missing = { $package } is not installed, so it cannot be reinstalled

# Variables:
#   $package (String) - Package name.
history-undo-version-missing = Undo is not supported for '{ $package }' because the prior version was not recorded

# Variables:
#   $package (String) - Package name.
history-undo-config-only = Undo is not supported for '{ $package }' because restoring config-files-only state is not implemented

# Variables:
#   $package (String) - Package name.
history-undo-installed-version-missing = Undo is not supported for '{ $package }' because the prior installed version was not recorded

# Variables:
#   $package (String) - Package name.
history-undo-reinstall = Undo is not supported for '{ $package }' because reinstall has no recorded inverse

# Variables:
#   $package (String) - Package name.
history-undo-held = Held package '{ $package }' cannot be undone

# Variables:
#   $package (String) - Package name.
history-redo-version-missing = Redo is not supported for '{ $package }' because the resulting version was not recorded

# Variables:
#   $package (String) - Package name.
history-redo-reinstall-missing = Redo is not supported for '{ $package }' because the reinstalled version was not recorded

# Variables:
#   $package (String) - Package name.
history-redo-held = Held package '{ $package }' cannot be redone

## History: labels

history-id = ID
history-command = Command
history-date-time = Date and Time
history-requested-by = Requested-By
history-altered = Altered
history-status = Status
history-started = Started
history-finished = Finished
history-targets = Requested Targets
history-none = None

## Show: general output

show-local-install = local install

# Variables:
#   $count (Number) - Number of additional package records.
#   $switch (String) - Command-line switch that displays every record.
show-more-records = There are { $count } additional records. Please use the { $switch } switch to see them.

show-unknown = Unknown
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
