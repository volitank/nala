#!/usr/bin/env python3
"""Exercise release publication without Debian tools or network access."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "scripts/ci-release-assets"
MOCK = """import json, os, pathlib, shutil, sys
command = pathlib.Path(sys.argv[0]).name
if command == 'dpkg-parsechangelog':
    print('0.18.0')
elif command == 'dpkg-deb':
    print({'Package': 'nala', 'Version': os.getenv('TEST_VERSION', '0.18.0'),
           'Architecture': os.getenv('TEST_ARCH', 'amd64')}[sys.argv[-1]])
else:
    if os.getenv('TEST_UPLOAD_FAIL'):
        sys.exit(22)
    source = pathlib.Path(sys.argv[sys.argv.index('--upload-file') + 1])
    shutil.copy(source, pathlib.Path('uploads') / source.name)
    with open('requests.jsonl', 'a') as log:
        log.write(json.dumps(sys.argv[1:]) + '\\n')
"""


class ReleaseAssets(unittest.TestCase):

	def setUp(self):
		self.temp = tempfile.TemporaryDirectory()
		self.addCleanup(self.temp.cleanup)
		self.root = Path(self.temp.name)
		(self.root / "bin").mkdir()
		for command in ("dpkg-parsechangelog", "dpkg-deb", "curl"):
			file = self.root / "bin" / command
			file.write_text(f"#!{sys.executable}\n{MOCK}")
			file.chmod(0o755)
		(self.root / "dist/debian").mkdir(parents=True)
		(self.root / "dist/debian/nala_0.18.0_amd64.deb").write_bytes(b"package")
		(self.root / "Cargo.toml").write_text('[package]\nversion = "0.18.0"\n')
		(self.root / "build.log").write_text("successful build\n")
		(self.root / "uploads").mkdir()
		(self.root / "tmp").mkdir()
		self.env = dict(os.environ, PATH=f"{self.root / 'bin'}:{os.environ['PATH']}",
			CI_COMMIT_TAG="v0.18.0", CI_JOB_TOKEN="test-token", CI_PROJECT_ID="123",
			CI_API_V4_URL="https://gitlab.example/api/v4", TMPDIR=str(self.root / "tmp"))

	def run_script(self, target="sid-amd64", **env):
		return subprocess.run(["sh", str(SCRIPT), target], cwd=self.root,
			env=self.env | env, capture_output=True, text=True)

	def test_branches_and_prereleases_do_not_publish(self):
		for tag in ("", "v0.18.0-rc1", "unrelated-tag", "v0x18x0"):
			with self.subTest(tag=tag):
				self.assertEqual(self.run_script(CI_COMMIT_TAG=tag).returncode, 0)
		self.assertEqual(list((self.root / "uploads").iterdir()), [])

	def test_all_targets_have_distinct_downloads_and_complete_bundles(self):
		(self.root / "source-build.log").write_text("successful source build\n")
		(self.root / "dist/debian/nala_0.18.0.dsc").write_text("source descriptor\n")
		for target in ("sid-amd64", "sid-arm64", "ubuntu-24.04-amd64"):
			with self.subTest(target=target):
				result = self.run_script(target, TEST_ARCH=target.split("-")[-1])
				self.assertEqual(result.returncode, 0, result.stderr)
				name = f"nala-v0.18.0-{target}"
				self.assertEqual((self.root / f"uploads/{name}.deb").read_bytes(), b"package")
				with tarfile.open(self.root / f"uploads/{name}.tar.gz") as archive:
					self.assertIn("dist/debian/nala_0.18.0.dsc", archive.getnames())
					self.assertIn("source-build.log", archive.getnames())
		requests = [json.loads(line) for line in (self.root / "requests.jsonl").read_text().splitlines()]
		self.assertEqual(len(requests), 6)
		self.assertEqual(len({args[-1] for args in requests}), 6)
		for args in requests:
			self.assertIn("--fail", args)
			self.assertIn("JOB-TOKEN: test-token", args)
			self.assertTrue(args[-1].startswith("https://gitlab.example/api/v4/projects/123/packages/generic/nala/v0.18.0/"))
		self.assertEqual(list((self.root / "tmp").iterdir()), [])

	def test_wrong_version_architecture_and_target_block_publication(self):
		for target, env in (("sid-amd64", {"CI_COMMIT_TAG": "v0.19.0"}),
			("sid-amd64", {"TEST_VERSION": "0.17.2"}),
			("sid-amd64", {"TEST_ARCH": "arm64"}), ("unknown", {})):
			with self.subTest(target=target, env=env):
				self.assertNotEqual(self.run_script(target, **env).returncode, 0)
		(self.root / "Cargo.toml").write_text('version = "0.17.2"\n')
		self.assertNotEqual(self.run_script().returncode, 0)
		self.assertEqual(list((self.root / "uploads").iterdir()), [])

	def test_failed_upload_fails_the_job_and_cleans_up(self):
		self.assertNotEqual(self.run_script(TEST_UPLOAD_FAIL="1").returncode, 0)
		self.assertEqual(list((self.root / "tmp").iterdir()), [])

	def test_missing_or_ambiguous_package_blocks_publication(self):
		package = self.root / "dist/debian/nala_0.18.0_amd64.deb"
		package.unlink()
		self.assertNotEqual(self.run_script().returncode, 0)
		package.write_bytes(b"package")
		(self.root / "dist/debian/nala_0.17.2_amd64.deb").write_bytes(b"old package")
		self.assertNotEqual(self.run_script().returncode, 0)
		self.assertEqual(list((self.root / "uploads").iterdir()), [])


if __name__ == "__main__":
	unittest.main()
