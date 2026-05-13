history-cleared-count =
    Cleared { $count } history { $count ->
        [one] entry
       *[other] entries
    }.
history-cleared-entry = Cleared history entry { $id }.
history-no-entries = No history entries found.
history-no-entries-error = No history entries found.
history-entry-not-found = History entry with ID '{ $id }' does not exist
history-entry-not-replayable = History entry '{ $id }' is not replayable because it was not recorded as applied
history-entry-no-package-changes = History entry '{ $id }' has no package changes to replay
history-version-not-found = Version '{ $version }' not found for '{ $package }'
history-reinstall-not-installed = { $package } is not installed, so it cannot be reinstalled
history-undo-prior-version-missing = Undo is not supported for '{ $package }' because the prior version was not recorded
history-undo-config-files-only = Undo is not supported for '{ $package }' because restoring config-files-only state is not implemented
history-undo-prior-installed-version-missing = Undo is not supported for '{ $package }' because the prior installed version was not recorded
history-undo-reinstall-no-inverse = Undo is not supported for '{ $package }' because reinstall has no recorded inverse
history-undo-held = Held package '{ $package }' cannot be undone
history-redo-result-version-missing = Redo is not supported for '{ $package }' because the resulting version was not recorded
history-redo-reinstall-version-missing = Redo is not supported for '{ $package }' because the reinstalled version was not recorded
history-redo-held = Held package '{ $package }' cannot be redone

history-table-id = ID
history-table-command = Command
history-table-date-time = Date and Time
history-table-requested-by = Requested-By
history-table-altered = Altered

history-detail-id = ID
history-detail-status = Status
history-detail-command = Command
history-detail-requested-by = Requested-By
history-detail-started = Started
history-detail-finished = Finished
history-detail-requested-targets = Requested Targets
history-detail-altered = Altered
history-detail-none = None
