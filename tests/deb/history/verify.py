#!/usr/bin/env python3

import json
import sys
from pathlib import Path


def transition(entry):
    package, = entry["packages"]
    return (
        entry["id"], package["name"], package["operation"],
        package["before"]["version"], package["after"]["version"],
    )


history_dir = Path(sys.argv[1])
paths = sorted(history_dir.glob("[0-9]*.json"), key=lambda path: int(path.stem))
entries = [json.loads(path.read_text()) for path in paths]
assert [path.stem for path in paths] == [str(i) for i in range(1, len(paths) + 1)]
assert [entry["id"] for entry in entries] == list(range(1, len(paths) + 1))
assert (history_dir / ".legacy-history-handled").is_file()
assert all(
    entry["schema_version"] == 1 and entry["status"] == "Applied"
    for entry in entries
)
legacy = entries[0]
assert legacy["command"] == "upgrade"
assert transition(legacy) == (
    1, "numeric-old-version", "Upgrade", "35", "37~deb12u1",
)
if rust := next(
    (entry for entry in entries if entry["command"] == "beta full-upgrade"), None
):
    assert "altered" not in rust
    assert transition(rust) == (
        6, "beta-package", "Upgrade", "1.0", "2.0",
    )
if installed := next(
    (
        entry
        for entry in entries
        if entry["requested_targets"] == ["nala-history-test=1.0"]
    ),
    None,
):
    assert transition(installed) == (
        7, "nala-history-test", "Install", None, "1.0",
    )
if expected := sys.argv[2:]:
    operations = [
        [package["operation"] for package in entry["packages"]] for entry in entries
    ]
    assert operations == [[operation] for operation in expected], operations

print("PASS: history files, IDs, and operations are canonical.")
