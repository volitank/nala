#!/usr/bin/env python3
"""Check that locale reruns preserve packaging flags and remain CI-only."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class PackagingTests(unittest.TestCase):
	def run_tests(self, upstream=False, guard=False, nocheck=False, profile=""):
		with tempfile.TemporaryDirectory() as tmp:
			base = Path(tmp)
			log = base / "calls"
			stub = base / "cargo"
			stub.write_text("#!/usr/bin/env python3\nimport json, os, sys\n"
				"with open(os.environ['CALL_LOG'], 'a') as f:\n"
				" f.write(json.dumps({'args':sys.argv[1:], 'locale':os.getenv('LC_ALL'), "
				"'apt':os.getenv('APT_CONFIG'), 'hardening':os.getenv('DEB_BUILD_MAINT_OPTIONS'), "
				"'cargo_home':os.getenv('CARGO_HOME'), 'debug':os.getenv('CARGO_PROFILE_LTO_DEBUG')})+'\\n')\n")
			stub.chmod(0o755)
			(base / "dh_auto_test").symlink_to(stub)
			env = dict(os.environ, PATH=f"{base}:{os.environ['PATH']}", CALL_LOG=str(log),
				NALA_CI_LOCALE_CHECK="1" if guard else "", NALA_CI_TEST_PROFILE=profile,
				DEB_BUILD_OPTIONS="nocheck" if nocheck else "terse",
				DEB_BUILD_PROFILES="upstream-cargo" if upstream else "", LC_ALL="C")
			env.pop("APT_CONFIG", None)
			env.pop("CARGO_HOME", None)
			subprocess.run(["make", "-f", "debian/rules", "override_dh_auto_test"],
				cwd=ROOT, env=env, check=True, capture_output=True)
			return [json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []

	def test_guard_reuses_command_and_build_settings(self):
		for upstream in (False, True):
			with self.subTest(upstream=upstream):
				normal, guard = self.run_tests(upstream=upstream, guard=True)
				self.assertEqual(normal["args"], guard["args"])
				for key in ("hardening", "cargo_home", "debug"):
					self.assertEqual(normal[key], guard[key])
				self.assertEqual(normal["hardening"], "hardening=+all")
				self.assertEqual(normal["locale"], "C")
				self.assertIsNone(normal["apt"])
				self.assertEqual(guard["locale"], "de_DE.UTF-8")
				self.assertEqual(guard["apt"], str(ROOT / "tests/empty-apt.conf"))
				if upstream:
					self.assertEqual(guard["debug"], "2")
					self.assertEqual(guard["cargo_home"], str(ROOT / "debian/cargo_home"))

	def test_archive_builds_do_not_add_the_guard(self):
		for upstream in (False, True):
			self.assertEqual(len(self.run_tests(upstream=upstream)), 1)

	def test_nocheck_skips_both_runs(self):
		for upstream in (False, True):
			self.assertEqual(self.run_tests(upstream=upstream, guard=True, nocheck=True), [])

	def test_release_test_profile_is_explicit_and_sid_only(self):
		normal, guard = self.run_tests(guard=True, profile="release")
		self.assertIn("--release", normal["args"])
		self.assertEqual(normal["args"], guard["args"])
		for profile in ("", "other"):
			self.assertNotIn("--release", self.run_tests(profile=profile)[0]["args"])
		self.assertNotIn("--release", self.run_tests(upstream=True, profile="release")[0]["args"])


if __name__ == "__main__":
	unittest.main()
