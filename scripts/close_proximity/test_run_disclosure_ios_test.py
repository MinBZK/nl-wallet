import json
import pathlib
import subprocess
import sys
import unittest
from unittest.mock import Mock, patch

from run_disclosure_ios_test import Runner


class WalletDisclosureRunnerTests(unittest.TestCase):
    def setUp(self):
        print_patch = patch("run_disclosure_ios_test.print", create=True)
        print_patch.start()
        self.addCleanup(print_patch.stop)
        self.runner = Runner(
            pathlib.Path(__file__).resolve().parents[2],
            "CLOSE_PROXIMITY_MAC_READER=",
            120,
            wallet_verifier=pathlib.Path(sys.executable),
        )
        self.addCleanup(self.runner.artifacts.cleanup)

    def pending_response(self, received=True):
        directory = pathlib.Path(self.runner.artifacts.name)
        expected = directory / "expected.json"
        response = directory / "received.json"
        expected.write_text("{}")
        if received:
            response.write_text("{}")
        self.runner.pending_verification = (expected, response)

    def test_signed_request_larger_than_old_log_window_launches_reader(self):
        payload = {
            "qr": "test-engagement",
            "device_request_hex": "ab" * 6000,
            "wallet_verification": {"issuer_trust_anchors": ["test-ca"]},
        }
        marker = "CLOSE_PROXIMITY_MAC_READER=" + json.dumps(payload)
        with patch("run_disclosure_ios_test.subprocess.Popen") as start:
            self.runner._maybe_start_helper(marker[:9000])
            start.assert_not_called()
            self.runner._maybe_start_helper(marker)
            command = start.call_args.args[0]
        self.assertIn("--device-response-file", command)
        expected, received = self.runner.pending_verification
        self.assertEqual(json.loads(expected.read_text()), payload["wallet_verification"])
        self.assertFalse(received.exists())

    def test_second_marker_is_not_hidden_by_first_marker_in_log_buffer(self):
        first = json.dumps({"qr": "first"})
        second = json.dumps({"qr": "second"})
        self.runner.seen_payloads.add(first)
        with patch("run_disclosure_ios_test.subprocess.Popen") as start:
            self.runner._maybe_start_helper(
                f"CLOSE_PROXIMITY_MAC_READER={first}\nCLOSE_PROXIMITY_MAC_READER={second}\n"
            )
        self.assertIn("second", start.call_args.args[0])

    def test_reader_success_and_verifier_failure_fail_overall_run(self):
        self.pending_response()
        with patch("run_disclosure_ios_test.subprocess.run", return_value=Mock(returncode=7)):
            self.runner._record_helper_result(0)
        self.assertEqual(self.runner.helper_exit_code, 7)
        self.assertEqual(self.runner.verified_wallet_tests, 0)

    def test_verified_response_counts_once(self):
        self.pending_response()
        with patch("run_disclosure_ios_test.subprocess.run", return_value=Mock(returncode=0)) as verify:
            self.runner._record_helper_result(0)
            self.runner._record_helper_result(0)
        verify.assert_called_once()
        self.assertEqual(self.runner.verified_wallet_tests, 1)
        self.assertEqual(self.runner.helper_exit_code, 0)

    def test_missing_response_artifact_fails(self):
        self.pending_response(received=False)
        with patch("run_disclosure_ios_test.subprocess.run") as verify:
            self.runner._record_helper_result(0)
        verify.assert_not_called()
        self.assertEqual(self.runner.helper_exit_code, 1)

    def test_reader_failure_is_preserved_without_running_verifier(self):
        self.pending_response()
        with patch("run_disclosure_ios_test.subprocess.run") as verify:
            self.runner._record_helper_result(3)
        verify.assert_not_called()
        self.assertEqual(self.runner.helper_exit_code, 3)

    def test_verifier_timeout_fails(self):
        self.pending_response()
        with patch("run_disclosure_ios_test.subprocess.run", side_effect=subprocess.TimeoutExpired("verifier", 30)):
            self.runner._record_helper_result(0)
        self.assertEqual(self.runner.helper_exit_code, 1)

    def test_successful_test_command_without_wallet_test_is_rejected_when_required(self):
        self.runner.require_wallet_test = True
        with patch("run_disclosure_ios_test.signal.signal"):
            result = self.runner.run([sys.executable, "-c", "pass"])
        self.assertEqual(result, 1)


if __name__ == "__main__":
    unittest.main()
