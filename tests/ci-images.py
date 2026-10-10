#!/usr/bin/env python3
"""Check native CI image builds, registry authentication and cleanup."""
import base64
import csv
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class ImageBuildTests(unittest.TestCase):
	def run_build(self, target="sid", arch="arm64", failure=False, missing=False):
		with tempfile.TemporaryDirectory() as tmp:
			base = Path(tmp)
			log = base / "call.json"
			stub = base / "buildctl-daemonless.sh"
			stub.write_text("#!/usr/bin/env python3\nimport json, os, sys\nfrom pathlib import Path\n"
				"config=Path(os.environ['DOCKER_CONFIG'])/'config.json'\n"
				"Path(os.environ['CALL_LOG']).write_text(json.dumps({'args':sys.argv[1:], "
				"'flags':os.environ['BUILDKITD_FLAGS'], 'config':str(config), "
				"'auth':json.loads(config.read_text()), 'mode':config.stat().st_mode & 0o777}))\n"
				"sys.exit(1 if os.environ.get('FAIL_BUILD') else 0)\n")
			stub.chmod(0o755)
			env = dict(os.environ, PATH=f"{base}:{os.environ['PATH']}", CALL_LOG=str(log),
				CI_PROJECT_DIR=str(base), CI_REGISTRY="registry.example.org",
				CI_REGISTRY_IMAGE="registry.example.org/volian/nala", CI_PIPELINE_ID="123",
				CI_REGISTRY_USER="ci-user", CI_REGISTRY_PASSWORD="ci-token",
				NALA_CI_IMAGE_VERSION="test-1", RUNNER_ARCH=arch, FAIL_BUILD="1" if failure else "")
			if missing:
				env.pop("CI_REGISTRY_PASSWORD")
			result = subprocess.run([str(ROOT / "scripts/ci-image"), target],
				env=env, capture_output=True, text=True)
			return result, json.loads(log.read_text()) if log.exists() else None

	def test_native_images_cache_layers_and_use_pipeline_tags(self):
		for target, arch in (("sid", "amd64"), ("sid", "arm64"), ("ubuntu", "amd64")):
			with self.subTest(target=target, arch=arch):
				result, call = self.run_build(target, arch)
				self.assertEqual(result.returncode, 0, result.stderr)
				args = call["args"]
				self.assertIn(f"platform=linux/{arch}", args)
				self.assertIn(f"target={target}", args)
				self.assertIn("build-arg:CI_IMAGE_VERSION=test-1", args)
				image = f"registry.example.org/volian/nala/ci/{target}-{arch}"
				self.assertIn(f"type=registry,ref={image}:cache-test-1", args)
				self.assertIn("type=inline", args)
				output = next(csv.reader([args[args.index("--output") + 1]]))
				self.assertIn(f"name={image}:123,{image}:cache-test-1", output)
				self.assertIn("push=true", output)
				self.assertIn("--oci-worker-no-process-sandbox", call["flags"])
				self.assertIn("--oci-worker-snapshotter=native", call["flags"])
				self.assertEqual(call["mode"], 0o600)
				auth = call["auth"]["auths"]["registry.example.org"]["auth"]
				self.assertEqual(base64.b64decode(auth), b"ci-user:ci-token")
				self.assertFalse(Path(call["config"]).parent.exists())
				self.assertNotIn("ci-token", result.stdout + result.stderr)

	def test_failed_build_cleans_registry_credentials(self):
		result, call = self.run_build(failure=True)
		self.assertNotEqual(result.returncode, 0)
		self.assertFalse(Path(call["config"]).parent.exists())

	def test_invalid_target_or_architecture_does_not_build(self):
		for target, arch in (("other", "amd64"), ("sid", "other"), ("ubuntu", "arm64")):
			result, call = self.run_build(target, arch)
			self.assertEqual(result.returncode, 2)
			self.assertIsNone(call)

	def test_missing_registry_credentials_does_not_build(self):
		result, call = self.run_build(missing=True)
		self.assertNotEqual(result.returncode, 0)
		self.assertIsNone(call)


if __name__ == "__main__":
	unittest.main()
