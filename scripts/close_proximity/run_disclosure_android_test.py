#!/usr/bin/env python3

import argparse
import json
import os
import pathlib
import re
import shlex
import signal
import subprocess
import sys
import threading
from typing import Optional


# Example usage from the nl_wallet root:
# scripts/close_proximity/run_disclosure_android_test.py -- ./gradlew connectedDebugAndroidTest -Pandroid.testInstrumentationRunnerArguments.class=nl.rijksoverheid.edi.wallet.platform_support.close_proximity_disclosure.CloseProximityDisclosureBridgeInstrumentedTest#test_close_proximity_disclosure_full_flow_with_mac_reader
def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run an Android close-proximity test command, watch adb logcat for the "
            "close-proximity host-helper marker, and launch the host Mac BLE helper automatically. "
            "If --serial is omitted, the script auto-selects the single connected Android device."
        )
    )
    parser.add_argument(
        "--serial",
        help="ADB serial to use. If omitted, auto-select the single connected Android device.",
    )
    parser.add_argument(
        "--helper-timeout",
        type=int,
        default=120,
        help="Timeout passed to disclosure_mac_reader.swift. Default: 120",
    )
    parser.add_argument(
        "--qr-marker",
        default="CLOSE_PROXIMITY_MAC_READER=",
        help="Machine-readable host-helper prefix emitted by the test. Default: CLOSE_PROXIMITY_MAC_READER=",
    )
    parser.add_argument(
        "--logcat-tag",
        default="CloseProximityTest",
        help="Logcat tag to follow for the QR marker. Default: CloseProximityTest",
    )
    parser.add_argument(
        "command",
        nargs=argparse.REMAINDER,
        help="Command to execute. Pass it after --, for example: -- ./gradlew ... connectedDebugAndroidTest",
    )
    args = parser.parse_args()

    if args.command and args.command[0] == "--":
        args.command = args.command[1:]

    if not args.command:
        parser.error("Missing command to run. Pass it after --.")

    return args


class Runner:
    def __init__(
        self,
        repo_root: pathlib.Path,
        serial: Optional[str],
        qr_marker: str,
        helper_timeout: int,
        logcat_tag: str,
    ) -> None:
        self.repo_root = repo_root
        self.serial = serial
        self.qr_pattern = re.compile(re.escape(qr_marker) + r"(\{[^\r\n]+\})")
        self.helper_timeout = helper_timeout
        self.logcat_tag = logcat_tag
        self.test_process: Optional[subprocess.Popen[str]] = None
        self.logcat_process: Optional[subprocess.Popen[str]] = None
        self.helper_process: Optional[subprocess.Popen[bytes]] = None
        self.helper_exit_code = 0
        self.logcat_exit_code = 0
        self.seen_payloads: set[str] = set()
        self._stopping = False
        self._lock = threading.Lock()

    def run(self, command: list[str]) -> int:
        serial = self.serial or self._detect_serial()
        command, test_cwd = self._normalize_test_command(command)
        environment = os.environ.copy()
        environment["ANDROID_SERIAL"] = serial

        print(
            f"[auto-close-proximity] Using Android device serial {serial}",
            file=sys.stderr,
            flush=True,
        )

        self._clear_logcat(serial)
        self.logcat_process = subprocess.Popen(
            [
                "adb",
                "-s",
                serial,
                "logcat",
                "-v",
                "brief",
                f"{self.logcat_tag}:I",
                "*:S",
            ],
            cwd=self.repo_root,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
        )
        try:
            self.test_process = subprocess.Popen(
                command,
                cwd=test_cwd,
                env=environment,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                bufsize=1,
            )
        except FileNotFoundError as error:
            self._stop_logcat()
            missing_binary = error.filename or command[0]
            raise SystemExit(
                "[auto-close-proximity] Failed to launch the test command. "
                f"Binary not found: {missing_binary}"
            ) from error

        signal.signal(signal.SIGINT, self._handle_signal)
        signal.signal(signal.SIGTERM, self._handle_signal)

        logcat_thread = threading.Thread(
            target=self._stream_logcat,
            name="close-proximity-logcat",
            daemon=True,
        )
        test_thread = threading.Thread(
            target=self._stream_test_output,
            name="close-proximity-test-output",
            daemon=True,
        )
        logcat_thread.start()
        test_thread.start()

        test_exit_code = self.test_process.wait()
        test_thread.join()

        self._stop_logcat()
        logcat_thread.join()
        helper_exit_code = self._finish_helper()

        if test_exit_code != 0:
            return test_exit_code
        if self.logcat_exit_code != 0:
            return self.logcat_exit_code
        if helper_exit_code != 0:
            return helper_exit_code
        return 0

    def _stream_test_output(self) -> None:
        if not self.test_process or not self.test_process.stdout:
            return

        for line in self.test_process.stdout:
            sys.stdout.write(line)
            sys.stdout.flush()

    def _stream_logcat(self) -> None:
        if not self.logcat_process or not self.logcat_process.stdout:
            return

        for line in self.logcat_process.stdout:
            sys.stderr.write(line)
            sys.stderr.flush()
            self._maybe_start_helper(line)

        exit_code = self.logcat_process.wait()
        if not self._stopping and exit_code != 0 and self.logcat_exit_code == 0:
            print(
                f"\n[auto-close-proximity] logcat exited with code {exit_code}\n",
                file=sys.stderr,
                flush=True,
            )
            self.logcat_exit_code = exit_code

    def _maybe_start_helper(self, text: str) -> None:
        match = self.qr_pattern.search(text)
        if not match:
            return

        payload_text = match.group(1)
        with self._lock:
            if payload_text in self.seen_payloads:
                return

            if self.helper_process and self.helper_process.poll() is None:
                return

            try:
                payload = json.loads(payload_text)
            except json.JSONDecodeError:
                return

            qr_code = payload.get("qr")
            if not isinstance(qr_code, str) or not qr_code:
                return

            self.seen_payloads.add(payload_text)
            helper_command = [
                "swift",
                "scripts/close_proximity/disclosure_mac_reader.swift",
                "--timeout",
                str(self.helper_timeout),
                "--qr-code",
                qr_code,
            ]
            device_request_hex = payload.get("device_request_hex")
            if isinstance(device_request_hex, str) and device_request_hex:
                helper_command.extend(["--device-request-hex", device_request_hex])

            reader_ca_crt_file = payload.get("reader_ca_crt_file")
            if isinstance(reader_ca_crt_file, str) and reader_ca_crt_file:
                helper_command.extend(["--reader-ca-crt-file", reader_ca_crt_file])

            reader_ca_key_file = payload.get("reader_ca_key_file")
            if isinstance(reader_ca_key_file, str) and reader_ca_key_file:
                helper_command.extend(["--reader-ca-key-file", reader_ca_key_file])

            reader_auth_file = payload.get("reader_auth_file")
            if isinstance(reader_auth_file, str) and reader_auth_file:
                helper_command.extend(["--reader-auth-file", reader_auth_file])

            expected_device_response_hex = payload.get("expected_device_response_hex")
            if isinstance(expected_device_response_hex, str) and expected_device_response_hex:
                helper_command.extend(
                    ["--expect-device-response-hex", expected_device_response_hex]
                )
            if payload.get("print_device_response_hex") is True:
                helper_command.append("--print-device-response-hex")
            command_string = " ".join(shlex.quote(part) for part in helper_command)
            print(
                f"\n[auto-close-proximity] Launching helper: {command_string}\n",
                file=sys.stderr,
                flush=True,
            )

            try:
                self.helper_process = subprocess.Popen(
                    helper_command,
                    cwd=self.repo_root,
                    stdout=sys.stderr.buffer,
                    stderr=sys.stderr.buffer,
                )
            except OSError as error:
                print(
                    f"\n[auto-close-proximity] Failed to launch helper: {error}\n",
                    file=sys.stderr,
                    flush=True,
                )
                if self.helper_exit_code == 0:
                    self.helper_exit_code = 1
                self.helper_process = None
                return

            threading.Thread(
                target=self._wait_for_helper,
                name="close-proximity-helper",
                daemon=True,
            ).start()

    def _wait_for_helper(self) -> None:
        helper_process = self.helper_process
        if not helper_process:
            return

        exit_code = helper_process.wait()
        with self._lock:
            print(
                f"\n[auto-close-proximity] Helper exited with code {exit_code}\n",
                file=sys.stderr,
                flush=True,
            )
            if exit_code != 0 and self.helper_exit_code == 0:
                self.helper_exit_code = exit_code
            if self.helper_process is helper_process:
                self.helper_process = None

    def _finish_helper(self) -> int:
        helper_process: Optional[subprocess.Popen[bytes]]
        with self._lock:
            helper_process = self.helper_process

        if not helper_process:
            return self.helper_exit_code

        try:
            helper_exit_code = helper_process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            helper_process.terminate()
            try:
                helper_exit_code = helper_process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                helper_process.kill()
                helper_exit_code = helper_process.wait()

        with self._lock:
            if helper_exit_code != 0 and self.helper_exit_code == 0:
                self.helper_exit_code = helper_exit_code
            if self.helper_process is helper_process:
                self.helper_process = None
        return self.helper_exit_code

    def _handle_signal(self, signum: int, _frame) -> None:
        if self._stopping:
            return

        self._stopping = True
        if self.test_process and self.test_process.poll() is None:
            self.test_process.terminate()
        if self.logcat_process and self.logcat_process.poll() is None:
            self.logcat_process.terminate()
        with self._lock:
            if self.helper_process and self.helper_process.poll() is None:
                self.helper_process.terminate()
        raise SystemExit(128 + signum)

    def _detect_serial(self) -> str:
        environment_serial = os.environ.get("ANDROID_SERIAL")
        if environment_serial:
            return environment_serial

        try:
            completed = subprocess.run(
                ["adb", "devices", "-l"],
                cwd=self.repo_root,
                capture_output=True,
                text=True,
                check=True,
            )
        except subprocess.CalledProcessError as error:
            raise SystemExit(
                f"[auto-close-proximity] Failed to detect connected Android devices: {error}"
            ) from error

        devices = self._parse_adb_devices(completed.stdout)
        online_devices = [device for device in devices if device["state"] == "device"]
        physical_devices = [
            device
            for device in online_devices
            if not device["serial"].startswith("emulator-")
        ]

        if len(physical_devices) == 1:
            return physical_devices[0]["serial"]
        if len(physical_devices) > 1:
            rendered_devices = ", ".join(
                device["serial"] for device in physical_devices
            )
            raise SystemExit(
                "[auto-close-proximity] Multiple connected physical Android devices found. "
                f"Pass --serial manually. Devices: {rendered_devices}"
            )
        if len(online_devices) == 1:
            return online_devices[0]["serial"]
        if not online_devices:
            raise SystemExit(
                "[auto-close-proximity] No connected Android device found. Connect a device or pass --serial."
            )

        rendered_devices = ", ".join(device["serial"] for device in online_devices)
        raise SystemExit(
            "[auto-close-proximity] Multiple connected Android devices found. "
            f"Pass --serial manually. Devices: {rendered_devices}"
        )

    def _clear_logcat(self, serial: str) -> None:
        try:
            subprocess.run(
                ["adb", "-s", serial, "logcat", "-c"],
                cwd=self.repo_root,
                capture_output=True,
                text=True,
                check=True,
            )
        except subprocess.CalledProcessError as error:
            print(
                "[auto-close-proximity] Failed to clear logcat; continuing with existing logs. "
                f"The helper may react to a stale QR marker. Error: {error}",
                file=sys.stderr,
                flush=True,
            )

    def _normalize_test_command(self, command: list[str]) -> tuple[list[str], pathlib.Path]:
        if not command:
            return command, self.repo_root

        normalized = command.copy()
        module_dir = self.repo_root / "wallet_core/wallet/platform_support/android"
        module_gradlew = module_dir / "gradlew"
        project_dir_flags = {"-p", "--project-dir"}
        has_project_dir = any(part in project_dir_flags for part in normalized)
        first_argument = pathlib.Path(normalized[0]).name

        if first_argument not in {"gradle", "gradlew"}:
            return normalized, self.repo_root
        if not module_gradlew.exists():
            return normalized, self.repo_root

        if has_project_dir:
            normalized[0] = str(module_gradlew)
            print(
                f"[auto-close-proximity] Rewriting {first_argument} to {module_gradlew}",
                file=sys.stderr,
                flush=True,
            )
            return normalized, self.repo_root

        normalized[0] = "./gradlew"
        print(
            "[auto-close-proximity] Running the test command from "
            f"{module_dir} with the module-local gradlew wrapper",
            file=sys.stderr,
            flush=True,
        )
        return normalized, module_dir

    @staticmethod
    def _parse_adb_devices(output: str) -> list[dict[str, str]]:
        devices: list[dict[str, str]] = []
        for raw_line in output.splitlines():
            line = raw_line.strip()
            if (
                not line
                or line.startswith("List of devices attached")
                or line.startswith("*")
            ):
                continue

            parts = line.split()
            if len(parts) < 2:
                continue

            devices.append(
                {
                    "serial": parts[0],
                    "state": parts[1],
                }
            )
        return devices

    def _stop_logcat(self) -> None:
        self._stopping = True
        if self.logcat_process and self.logcat_process.poll() is None:
            self.logcat_process.terminate()
            try:
                self.logcat_process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.logcat_process.kill()
                self.logcat_process.wait()


def main() -> int:
    args = parse_args()
    repo_root = pathlib.Path(__file__).resolve().parent.parent.parent
    runner = Runner(
        repo_root=repo_root,
        serial=args.serial,
        qr_marker=args.qr_marker,
        helper_timeout=args.helper_timeout,
        logcat_tag=args.logcat_tag,
    )
    return runner.run(args.command)


if __name__ == "__main__":
    sys.exit(main())
