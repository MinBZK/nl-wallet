import json
import os
import pathlib
import select
import signal
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
        reader_print_patch = patch("disclosure_test_reader.print", create=True)
        reader_print_patch.start()
        self.addCleanup(reader_print_patch.stop)
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

    def test_native_suite_does_not_build_wallet_verifier(self):
        self.runner.wallet_verifier = None
        with patch("run_disclosure_ios_test.signal.signal"), patch("disclosure_test_reader.subprocess.run") as build:
            self.assertEqual(self.runner.run([sys.executable, "-c", "pass"]), 0)
            build.assert_not_called()

    def test_prebuilt_wallet_verifier_is_validated_without_building(self):
        self.runner.wallet_verifier = pathlib.Path(self.runner.artifacts.name) / "missing-verifier"
        with patch("disclosure_test_reader.subprocess.run") as build:
            with self.assertRaises(FileNotFoundError):
                self.runner.run([sys.executable, "-c", "pass"])
            build.assert_not_called()
        self.assertIsNone(self.runner.test_process)

    def test_required_verifier_build_failure_prevents_test_command_start(self):
        self.runner.require_wallet_test = True
        self.runner.wallet_verifier = None
        with patch(
            "disclosure_test_reader.subprocess.run", side_effect=subprocess.CalledProcessError(1, "cargo"),
        ) as build:
            with self.assertRaises(subprocess.CalledProcessError):
                self.runner.run([sys.executable, "-c", "pass"])
            build.assert_called_once()
        self.assertIsNone(self.runner.test_process)

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

    def test_reader_failure_between_polls_is_preserved(self):
        self.pending_response()
        pending = self.runner.pending_verification
        reader = Mock()
        reader.poll.side_effect = [None, 3]
        self.runner.helper_process = reader
        marker = "CLOSE_PROXIMITY_MAC_READER=" + json.dumps({"qr": "next"})

        with patch("run_disclosure_ios_test.subprocess.Popen") as start, \
                patch("run_disclosure_ios_test.subprocess.run") as verify:
            start.return_value.wait.return_value = 0
            self.runner._poll_helper()
            self.runner._maybe_start_helper(marker)
            start.assert_not_called()
            self.assertEqual(self.runner.pending_verification, pending)

            self.runner._poll_helper()
            self.runner._maybe_start_helper(marker)
            start.assert_called_once()
            self.assertEqual(self.runner._finish_helper(), 3)
            verify.assert_not_called()

    def test_reader_response_is_verified_before_starting_next_reader(self):
        self.pending_response()
        expected, received = self.runner.pending_verification
        reader = Mock()
        reader.poll.side_effect = [None, 0]
        self.runner.helper_process = reader
        marker = "CLOSE_PROXIMITY_MAC_READER=" + json.dumps({"qr": "next", "wallet_verification": {}})

        with patch("run_disclosure_ios_test.subprocess.Popen") as start, \
                patch("run_disclosure_ios_test.subprocess.run", return_value=Mock(returncode=7)) as verify:
            start.return_value.wait.return_value = 0
            self.runner._poll_helper()
            self.runner._maybe_start_helper(marker)
            start.assert_not_called()
            self.assertEqual(self.runner.pending_verification, (expected, received))

            self.runner._poll_helper()
            verify.assert_called_once_with(
                [str(self.runner.wallet_verifier), str(expected), str(received)],
                cwd=self.runner.repo_root,
                timeout=30,
                check=False,
            )
            self.runner._maybe_start_helper(marker)
            start.assert_called_once()
            self.runner.pending_verification[1].write_text("{}")
            verify.return_value.returncode = 0
            self.assertEqual(self.runner._finish_helper(), 7)
            self.assertEqual(verify.call_count, 2)
            self.assertEqual(self.runner.verified_wallet_tests, 1)

    def test_invalid_markers_do_not_hide_later_valid_marker(self):
        payloads = [
            "{invalid json}",
            json.dumps({"request_doc_type": "missing-qr"}),
            json.dumps({"qr": None}),
            json.dumps({"qr": 123}),
            json.dumps({"qr": ""}),
            json.dumps({"qr": "valid"}),
        ]
        buffer = "\n".join("CLOSE_PROXIMITY_MAC_READER=" + payload for payload in payloads)
        with patch("run_disclosure_ios_test.subprocess.Popen") as start:
            self.runner._maybe_start_helper(buffer)
            start.assert_called_once()
            self.assertIn("valid", start.call_args.args[0])

    def test_incomplete_nested_marker_is_processed_when_completed(self):
        marker = "CLOSE_PROXIMITY_MAC_READER=" + json.dumps({
            "qr": "test-engagement",
            "wallet_verification": {"issuer_trust_anchors": ["test-ca"]},
        })
        with patch("run_disclosure_ios_test.subprocess.Popen") as start:
            self.runner._maybe_start_helper(marker[:-1])
            start.assert_not_called()
            self.runner._maybe_start_helper(marker)
            start.assert_called_once()
            self.assertIn("test-engagement", start.call_args.args[0])

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

    def test_invalid_verification_metadata_stops_test_command_and_cleans_artifacts(self):
        marker = "CLOSE_PROXIMITY_MAC_READER=" + json.dumps({"qr": "test", "wallet_verification": []})
        command = [sys.executable, "-c", f"import time; print({marker!r}, flush=True); time.sleep(60)"]
        try:
            with patch("run_disclosure_ios_test.signal.signal"), patch("run_disclosure_ios_test.sys.stdout"):
                with self.assertRaisesRegex(RuntimeError, "Wallet disclosure requires verification metadata"):
                    self.runner.run(command)
            self.assertIsNotNone(self.runner.test_process.poll())
            self.assertFalse(pathlib.Path(self.runner.artifacts.name).exists())
        finally:
            if self.runner.test_process is not None:
                self.runner.test_process.kill()
                self.runner.test_process.wait()

    def test_wallet_marker_without_verifier_fails(self):
        self.runner.wallet_verifier = None
        marker = "CLOSE_PROXIMITY_MAC_READER=" + json.dumps({"qr": "test", "wallet_verification": {}})
        command = [sys.executable, "-c", f"import time; print({marker!r}, flush=True); time.sleep(60)"]
        try:
            with patch("run_disclosure_ios_test.signal.signal"), patch("run_disclosure_ios_test.sys.stdout"):
                with self.assertRaisesRegex(RuntimeError, "Wallet disclosure requires verification metadata"):
                    self.runner.run(command)
            self.assertIsNotNone(self.runner.test_process.poll())
            self.assertFalse(pathlib.Path(self.runner.artifacts.name).exists())
        finally:
            if self.runner.test_process is not None:
                self.runner.test_process.kill()
                self.runner.test_process.wait()

    def test_exception_stops_helper_descendants_even_when_they_ignore_termination(self):
        child_code = (
            "import os, signal, time; signal.signal(signal.SIGTERM, signal.SIG_IGN); "
            "print(os.getpid(), flush=True); time.sleep(60)"
        )
        parent_code = (
            f"import subprocess, sys, time; subprocess.Popen([sys.executable, '-c', {child_code!r}]); "
            "time.sleep(60)"
        )
        helper = subprocess.Popen(
            [sys.executable, "-c", parent_code], stdout=subprocess.PIPE, text=True, start_new_session=True,
        )
        self.runner.helper_process = helper
        child_pid = None
        try:
            self.assertTrue(select.select([helper.stdout], [], [], 5)[0], "Helper child did not start")
            child_pid = int(helper.stdout.readline())
            with patch("run_disclosure_ios_test.signal.signal"), \
                    patch.object(self.runner, "_poll_helper", side_effect=RuntimeError("monitor failed")):
                with self.assertRaisesRegex(RuntimeError, "monitor failed"):
                    self.runner.run([sys.executable, "-c", "import time; time.sleep(60)"])
            self.assertIsNotNone(self.runner.test_process.poll())
            self.assertIsNotNone(helper.poll())
            self.assertTrue(select.select([helper.stdout], [], [], 5)[0], "Helper child still holds stdout open")
            self.assertEqual(helper.stdout.read(), "")
            self.assertFalse(pathlib.Path(self.runner.artifacts.name).exists())
        finally:
            if child_pid is not None:
                try:
                    os.kill(child_pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
            helper.kill()
            helper.wait()
            helper.stdout.close()
            if self.runner.test_process is not None:
                self.runner.test_process.kill()
                self.runner.test_process.wait()

    def test_successful_test_command_without_wallet_test_is_rejected_when_required(self):
        self.runner.require_wallet_test = True
        with patch("run_disclosure_ios_test.signal.signal"):
            result = self.runner.run([sys.executable, "-c", "pass"])
        self.assertEqual(result, 1)


if __name__ == "__main__":
    unittest.main()
