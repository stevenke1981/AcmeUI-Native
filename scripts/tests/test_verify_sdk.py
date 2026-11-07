import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location("verify_sdk", Path(__file__).resolve().parents[1] / "verify_sdk.py")
VERIFY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VERIFY)


class EvidenceTests(unittest.TestCase):
    def suite(self, commands, dry_run=False, timeout=10):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            return VERIFY.run_suite(commands, root, root / "evidence", timeout, dry_run)

    def test_success_contains_log_hash(self):
        report = self.suite([[sys.executable, "-c", "print('evidence')"]])
        self.assertEqual(report["status"], "PASS")
        self.assertEqual(report["checks"][0]["exit_code"], 0)
        self.assertEqual(len(report["checks"][0]["log_sha256"]), 64)

    def test_failure_stops_later_checks(self):
        report = self.suite([[sys.executable, "-c", "raise SystemExit(7)"], [sys.executable, "-c", "print('must not run')"]])
        self.assertEqual(report["status"], "FAIL")
        self.assertEqual(report["checks"][0]["exit_code"], 7)
        self.assertEqual(report["checks"][1]["status"], "NOT_RUN")

    def test_missing_tool_is_blocked(self):
        report = self.suite([["acme-no-such-executable-7fb42"]])
        self.assertEqual(report["status"], "BLOCKED")

    def test_dry_run_never_claims_pass(self):
        report = self.suite([["acme-no-such-executable-7fb42"]], dry_run=True)
        self.assertEqual(report["status"], "NOT_RUN")

    def test_timeout_fails(self):
        report = self.suite([[sys.executable, "-c", "import time; time.sleep(10)"]], timeout=0.05)
        self.assertEqual(report["status"], "FAIL")
        self.assertEqual(report["checks"][0]["reason"], "Timeout")

    def test_does_not_overwrite_previous_evidence(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaises(FileExistsError):
                VERIFY.run_suite([], root, root, 10)

    def test_build_and_test_commands_are_locked(self):
        for commands in VERIFY.profiles().values():
            for command in commands:
                if command[1] != "fmt":
                    self.assertIn("--locked", command)


if __name__ == "__main__":
    unittest.main()
