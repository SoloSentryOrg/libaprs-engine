#!/usr/bin/env python3
"""Verify merge-gate decisions against unordered/retried checks without network."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class RerunTests(unittest.TestCase):
    def evaluate(self, runs):
        with tempfile.TemporaryDirectory(prefix="merge-gate-tests-") as directory:
            fake = Path(directory) / "gh"
            fake.write_text('#!/bin/sh\nset -eu\nprintf "%s" "$CHECKS_JSON" | jq -r "$4"\n')
            fake.chmod(0o700)
            checks = [dict(run, name=name) for name in ["Secret Scan", "Docs"] for run in runs]
            env = {**os.environ, "PATH": directory + os.pathsep + os.environ["PATH"],
                   "CHECKS_JSON": json.dumps({"check_runs": checks}),
                   "MERGE_GATE_CHANGED_FILES": "README.md", "MERGE_GATE_TIMEOUT_SECONDS": "0",
                   "GITHUB_REPOSITORY": "test/example", "HEAD_SHA": "a" * 40}
            return subprocess.run(["sh", "scripts/check-merge-gate.sh"], cwd=ROOT,
                                  env=env, capture_output=True, text=True, timeout=10).returncode

    def test_latest_failure_overrides_old_success(self):
        for order in [False, True]:
            checks = [{"id": 1, "status": "completed", "conclusion": "success", "app": {"id": 15368}},
                      {"id": 2, "status": "completed", "conclusion": "failure", "app": {"id": 15368}}]
            self.assertNotEqual(self.evaluate(list(reversed(checks)) if order else checks), 0)

    def test_latest_success_overrides_old_failure(self):
        self.assertEqual(self.evaluate([
            {"id": 2, "status": "completed", "conclusion": "success", "app": {"id": 15368}},
            {"id": 1, "status": "completed", "conclusion": "failure", "app": {"id": 15368}}]), 0)

    def test_latest_pending_does_not_reuse_old_success(self):
        self.assertNotEqual(self.evaluate([
            {"id": 2, "status": "in_progress", "conclusion": None, "app": {"id": 15368}},
            {"id": 1, "status": "completed", "conclusion": "success", "app": {"id": 15368}}]), 0)

    def test_other_producer_cannot_satisfy_gate(self):
        self.assertNotEqual(self.evaluate([
            {"id": 3, "status": "completed", "conclusion": "success", "app": {"id": 1}}]), 0)

    def test_missing_checks_fail(self):
        self.assertNotEqual(self.evaluate([]), 0)


if __name__ == "__main__":
    unittest.main()
