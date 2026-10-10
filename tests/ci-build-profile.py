#!/usr/bin/env python3
"""Check branch/tag compiler settings without compiling or publishing."""
import json
import os
from pathlib import Path
import subprocess
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "scripts/ci-build-profile"


class BuildProfile(unittest.TestCase):
	def test_branch_and_tag_settings(self):
		for tag in ("", "v0.18.1", "v0.18.1-rc1"):
			with self.subTest(tag=tag):
				result = subprocess.run(
					["sh", "-ec", '. "$1"; python3 -c "import json, os; print(json.dumps(dict(os.environ)))"', "sh", str(SCRIPT)],
					env=dict(os.environ, CI_COMMIT_TAG=tag), capture_output=True, text=True,
					check=True,
				)
				env = json.loads(result.stdout.splitlines()[-1])
				for profile in ("RELEASE", "LTO"):
					for setting, expected in {
						"OPT_LEVEL": "s" if tag else "0",
						"LTO": "true" if tag else "off",
						"DEBUG_ASSERTIONS": "false" if tag else "true",
						"OVERFLOW_CHECKS": "false" if tag else "true",
					}.items():
						self.assertEqual(env[f"CARGO_PROFILE_{profile}_{setting}"], expected)


if __name__ == "__main__":
	unittest.main()
