import base64
import contextlib
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

from run_disclosure_android_test import ChunkedMarker, Runner


MARKER = "CLOSE_PROXIMITY_MAC_READER="


def chunk_lines(payload, identifier="test"):
    encoded = base64.b64encode((MARKER + json.dumps(payload)).encode()).decode()
    chunks = [encoded[i:i + 2000] for i in range(0, len(encoded), 2000)]
    return [
        f"I/CloseProximityTest(123): {ChunkedMarker.PREFIX}{identifier}:{i}:{len(chunks)}:{chunk}"
        for i, chunk in enumerate(chunks)
    ]


class ChunkedMarkerTests(unittest.TestCase):
    def test_large_signed_request_survives_out_of_order_logcat_chunks(self):
        payload = {"qr": "engagement", "device_request_hex": "ab" * 6000, "wallet_verification": {}}
        decoder = ChunkedMarker(MARKER)
        lines = chunk_lines(payload)
        self.assertGreater(len(lines), 1)
        for line in reversed(lines[1:]):
            self.assertIsNone(decoder.consume(line))
        self.assertEqual(decoder.consume(lines[0]), MARKER + json.dumps(payload))
        self.assertFalse(decoder.pending)

    def test_missing_chunk_does_not_produce_a_reader_request(self):
        decoder = ChunkedMarker(MARKER)
        lines = chunk_lines({"qr": "engagement", "device_request_hex": "ab" * 6000})
        for line in lines[:-1]:
            self.assertIsNone(decoder.consume(line))
        self.assertTrue(decoder.pending)

    def test_corrupt_chunks_fail(self):
        for value in ("test:1:1:YWJj", "test:0:513:YWJj", "test:0:1:a", "test:0:1:YWJj", "bad"):
            with self.subTest(value=value):
                with self.assertRaises(RuntimeError):
                    ChunkedMarker(MARKER).consume(ChunkedMarker.PREFIX + value)

    def test_conflicting_chunk_fails(self):
        decoder = ChunkedMarker(MARKER)
        decoder.consume(ChunkedMarker.PREFIX + "test:0:2:YWJj")
        with self.assertRaisesRegex(RuntimeError, "Conflicting"):
            decoder.consume(ChunkedMarker.PREFIX + "test:0:2:ZGVm")

    def test_native_bridge_marker_is_unchanged(self):
        line = 'I/CloseProximityTest(123): ' + MARKER + '{"qr":"native-bridge"}'
        self.assertEqual(ChunkedMarker(MARKER).consume(line), line)


class AndroidWalletRunnerTests(unittest.TestCase):
    def setUp(self):
        self.runner = Runner(
            pathlib.Path(__file__).resolve().parents[2], "test-device", MARKER, 120,
            "CloseProximityTest", wallet_verifier=pathlib.Path(sys.executable), require_wallet_test=True,
        )
        self.addCleanup(self.runner.artifacts.cleanup)
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.done = pathlib.Path(self.directory.name) / "done"

    def waiting_test(self):
        # A deadline keeps a broken runner from hanging the host test suite.
        return [sys.executable, "-c", f"""
import pathlib, sys, time
done = pathlib.Path({str(self.done)!r})
deadline = time.monotonic() + 5
while not done.exists() and time.monotonic() < deadline:
    time.sleep(0.02)
sys.exit(0 if done.exists() else 99)
"""]

    @contextlib.contextmanager
    def processes(self, lines, reader_count=1, reader_exit=0, verifier_exit=0, response=True, logcat_exit=False):
        original_popen = subprocess.Popen
        readers = []

        def start(command, **kwargs):
            if command[0] == "adb":
                program = f"import time; print({chr(10).join(lines)!r}, flush=True)"
                if not logcat_exit:
                    program += "; time.sleep(60)"
                command = [sys.executable, "-c", program]
            elif command[0] == "swift":
                readers.append(command)
                received = command[command.index("--device-response-file") + 1]
                program = "import pathlib, sys, time; time.sleep(0.05); "
                if response:
                    program += f"pathlib.Path({received!r}).write_text('{{}}'); "
                if len(readers) == reader_count:
                    program += f"pathlib.Path({str(self.done)!r}).touch(); "
                program += f"sys.exit({reader_exit})"
                command = [sys.executable, "-c", program]
            return original_popen(command, **kwargs)

        with open(os.devnull, "w") as output, \
                patch.object(self.runner, "_clear_logcat"), \
                patch("run_disclosure_android_test.signal.signal"), \
                patch("run_disclosure_android_test.sys.stdout", output), \
                patch("run_disclosure_android_test.sys.stderr", output), \
                patch("run_disclosure_android_test.subprocess.Popen", side_effect=start), \
                patch("disclosure_test_reader.subprocess.run", return_value=Mock(returncode=verifier_exit)) as verify:
            yield readers, verify

    def assert_cleaned_up(self):
        self.assertIsNotNone(self.runner.test_process.poll())
        self.assertIsNotNone(self.runner.logcat_process.poll())
        self.assertFalse(pathlib.Path(self.runner.artifacts.name).exists())

    def test_queued_wallet_readers_are_both_verified(self):
        lines = chunk_lines({"qr": "first", "wallet_verification": {}}, "first")
        lines += chunk_lines({"qr": "second", "wallet_verification": {}}, "second")
        with self.processes(lines, reader_count=2) as (readers, verify):
            self.assertEqual(self.runner.run(self.waiting_test()), 0)
            self.assertEqual(len(readers), 2)
            self.assertEqual(verify.call_count, 2)
            self.assertEqual(self.runner.verified_wallet_tests, 2)
        self.assert_cleaned_up()

    def test_successful_instrumentation_cannot_hide_reader_or_verifier_failure(self):
        lines = chunk_lines({"qr": "test", "wallet_verification": {}})
        with self.processes(lines, verifier_exit=7) as (_, verify):
            self.assertEqual(self.runner.run(self.waiting_test()), 7)
            verify.assert_called_once()
        self.assert_cleaned_up()

    def test_reader_failure_is_not_verified(self):
        lines = chunk_lines({"qr": "test", "wallet_verification": {}})
        with self.processes(lines, reader_exit=3) as (_, verify):
            self.assertEqual(self.runner.run(self.waiting_test()), 3)
            verify.assert_not_called()
        self.assert_cleaned_up()

    def test_missing_response_artifact_fails(self):
        lines = chunk_lines({"qr": "test", "wallet_verification": {}})
        with self.processes(lines, response=False) as (_, verify):
            self.assertEqual(self.runner.run(self.waiting_test()), 1)
            verify.assert_not_called()
        self.assert_cleaned_up()

    def test_skipped_wallet_tests_fail(self):
        with self.processes([]):
            self.assertEqual(self.runner.run([sys.executable, "-c", "pass"]), 1)
        self.assert_cleaned_up()

    def test_native_suite_does_not_build_wallet_verifier(self):
        self.runner.require_wallet_test = False
        self.runner.wallet_verifier = None
        with self.processes([]), patch.object(self.runner, "_prepare_wallet_verifier") as prepare:
            self.assertEqual(self.runner.run([sys.executable, "-c", "pass"]), 0)
            prepare.assert_not_called()

    def test_invalid_metadata_stops_test_and_logcat(self):
        lines = chunk_lines({"qr": "test", "wallet_verification": []})
        with self.processes(lines):
            with self.assertRaisesRegex(RuntimeError, "verification metadata"):
                self.runner.run(self.waiting_test())
        self.assert_cleaned_up()

    def test_lost_logcat_fails_and_stops_test(self):
        with self.processes([], logcat_exit=True):
            with self.assertRaisesRegex(RuntimeError, "logcat stopped"):
                self.runner.run(self.waiting_test())
        self.assert_cleaned_up()

    def test_explicit_wallet_project_is_preserved(self):
        command = ["wallet_app/android/gradlew", "-p", "wallet_app/android/wallet-integration-tests", ":connectedDebugAndroidTest"]
        self.assertEqual(self.runner._normalize_test_command(command), (command, self.runner.repo_root))

    def test_implicit_gradlew_keeps_native_platform_support_project(self):
        command, directory = self.runner._normalize_test_command(["./gradlew", "connectedDebugAndroidTest"])
        self.assertEqual(command, ["./gradlew", "connectedDebugAndroidTest"])
        self.assertEqual(directory, self.runner.repo_root / "wallet_core/wallet/platform_support/android")


if __name__ == "__main__":
    unittest.main()
