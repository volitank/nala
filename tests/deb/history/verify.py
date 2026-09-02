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


args = sys.argv[1:]
native = args[:1] == ["--native"]
if native:
    args = args[1:]

history_dir = Path(args[0])
paths = sorted(history_dir.glob("[0-9]*.json"), key=lambda path: int(path.stem))
entries = [json.loads(path.read_text()) for path in paths]
assert [path.stem for path in paths] == [str(i) for i in range(1, len(paths) + 1)]
assert [entry["id"] for entry in entries] == list(range(1, len(paths) + 1))
assert all(
    entry["schema_version"] == 1 and entry["status"] == "Applied"
    for entry in entries
)

marker = history_dir / ".legacy-history-handled"
if native:
    assert not marker.exists()
else:
    assert marker.is_file()
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
if expected := args[1:]:
    operations = [
        [package["operation"] for package in entry["packages"]] for entry in entries
    ]
    assert operations == [[operation] for operation in expected], operations

print("PASS: history files, IDs, and operations are canonical.")
