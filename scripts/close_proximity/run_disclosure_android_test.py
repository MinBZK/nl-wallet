#!/usr/bin/env python3

import argparse
import base64
import binascii
import os
import pathlib
import re
import selectors
import signal
import subprocess
import sys
from typing import Optional

from disclosure_test_reader import Reader


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
        "--wallet-verifier",
        type=pathlib.Path,
        help="Prebuilt verify_close_proximity_response executable (otherwise built with --require-wallet-test).",
    )
    parser.add_argument(
        "--require-wallet-test",
        action="store_true",
        help="Build the host verifier and fail unless at least one Rust wallet disclosure is verified.",
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


class ChunkedMarker:
    """Reassemble test markers split below Android logcat's per-entry size limit."""

    PREFIX = "CLOSE_PROXIMITY_MAC_READER_CHUNK="
    PATTERN = re.compile(re.escape(PREFIX) + r"([a-zA-Z0-9-]+):(\d+):(\d+):([a-zA-Z0-9+/=]+)\s*$")

    def __init__(self, marker: str) -> None:
        self.marker = marker
        self.pending: dict[str, tuple[int, dict[int, str]]] = {}

    def consume(self, line: str) -> Optional[str]:
        if self.PREFIX not in line:
            return line
        match = self.PATTERN.search(line)
        if not match:
            raise RuntimeError("Malformed logcat reader chunk")
        identifier, index_text, count_text, chunk = match.groups()
        index, count = int(index_text), int(count_text)
        if not 0 <= index < count <= 512 or len(chunk) > 2000:
            raise RuntimeError("Invalid logcat reader chunk size or index")
        expected_count, parts = self.pending.setdefault(identifier, (count, {}))
        if count != expected_count or (index in parts and parts[index] != chunk):
            raise RuntimeError("Conflicting logcat reader chunks")
        parts[index] = chunk
        if len(parts) != count:
            return None
        del self.pending[identifier]
        try:
            marker = base64.b64decode("".join(parts[i] for i in range(count)), validate=True).decode("utf-8")
        except (binascii.Error, UnicodeDecodeError) as error:
            raise RuntimeError("Invalid base64 reader marker") from error
        if not marker.startswith(self.marker) or "\n" in marker or "\r" in marker:
            raise RuntimeError("Invalid reassembled reader marker")
        return marker


class Runner(Reader):
    def __init__(
        self,
        repo_root: pathlib.Path,
        serial: Optional[str],
        qr_marker: str,
        helper_timeout: int,
        logcat_tag: str,
        wallet_verifier: Optional[pathlib.Path] = None,
        require_wallet_test: bool = False,
    ) -> None:
        super().__init__(repo_root, qr_marker, helper_timeout, wallet_verifier, require_wallet_test)
        self.serial = serial
        self.logcat_tag = logcat_tag
        self.test_process: Optional[subprocess.Popen[bytes]] = None
        self.logcat_process: Optional[subprocess.Popen[bytes]] = None
        self.chunks = ChunkedMarker(qr_marker)
        self._stopping = False

    def run(self, command: list[str]) -> int:
        try:
            return self._run(command)
        except BaseException:
            try:
                if self.test_process is not None:
                    self._stop_process_group(self.test_process)
            finally:
                if self.helper_process is not None:
                    self._stop_process_group(self.helper_process)
            raise
        finally:
            try:
                if self.logcat_process is not None:
                    self._stop_process_group(self.logcat_process)
            finally:
                for process in (self.test_process, self.logcat_process):
                    if process is not None and process.stdout is not None:
                        process.stdout.close()
                self.artifacts.cleanup()

    def _run(self, command: list[str]) -> int:
        # Native PlatformSupport tests need no wallet verifier. Wallet tests prepare it before BLE starts.
        if self.require_wallet_test or self.wallet_verifier is not None:
            self._prepare_wallet_verifier()
        serial = self.serial or self._detect_serial()
        command, test_cwd = self._normalize_test_command(command)
        environment = os.environ.copy()
        environment["ANDROID_SERIAL"] = serial
        print(f"[auto-close-proximity] Using Android device serial {serial}", file=sys.stderr, flush=True)
        self._clear_logcat(serial)
        self.logcat_process = subprocess.Popen(
            ["adb", "-s", serial, "logcat", "-v", "brief", f"{self.logcat_tag}:I", "*:S"],
            cwd=self.repo_root,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            start_new_session=True,
        )
        self.test_process = subprocess.Popen(
            command,
            cwd=test_cwd,
            env=environment,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            start_new_session=True,
        )
        signal.signal(signal.SIGINT, self._handle_signal)
        signal.signal(signal.SIGTERM, self._handle_signal)

        buffer = ""
        partial_line = b""
        test_output_open = True
        with selectors.DefaultSelector() as selector:
            selector.register(self.test_process.stdout, selectors.EVENT_READ, "test")
            selector.register(self.logcat_process.stdout, selectors.EVENT_READ, "logcat")
            while test_output_open or self.test_process.poll() is None:
                self._poll_helper()
                # Retain markers while the previous reader finishes, including while Gradle is silent.
                self._maybe_start_helper(buffer)
                for key, _ in selector.select(timeout=0.1):
                    chunk = os.read(key.fd, 65536)
                    if not chunk:
                        selector.unregister(key.fileobj)
                        if key.data == "logcat":
                            raise RuntimeError(f"logcat stopped before the tests (exit {self.logcat_process.wait()})")
                        test_output_open = False
                        continue
                    output = sys.stdout if key.data == "test" else sys.stderr
                    output.write(chunk.decode("utf-8", errors="replace"))
                    output.flush()
                    if key.data == "logcat":
                        partial_line += chunk
                        while b"\n" in partial_line:
                            line, partial_line = partial_line.split(b"\n", 1)
                            marker = self.chunks.consume(line.decode("utf-8", errors="replace"))
                            if marker is not None:
                                buffer = (buffer + marker + "\n")[-1_048_576:]

        test_exit_code = self.test_process.wait()
        helper_exit_code = self._finish_helper()
        if test_exit_code != 0:
            return test_exit_code
        if self.chunks.pending:
            raise RuntimeError("Incomplete reader marker in logcat; a chunk was lost")
        if helper_exit_code != 0:
            return helper_exit_code
        if self.require_wallet_test and self.verified_wallet_tests == 0:
            print("No Rust wallet disclosure was verified; check the test selection.", file=sys.stderr)
            return 1
        return 0

    def _handle_signal(self, signum: int, _frame) -> None:
        if self._stopping:
            return
        self._stopping = True
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
        # A stale engagement must never launch a reader for the current test.
        subprocess.run(
            ["adb", "-s", serial, "logcat", "-c"],
            cwd=self.repo_root,
            capture_output=True,
            text=True,
            check=True,
        )

    def _normalize_test_command(self, command: list[str]) -> tuple[list[str], pathlib.Path]:
        if not command:
            return command, self.repo_root

        normalized = command.copy()
        # Explicit wrappers (including the wallet project's standalone test host) keep their project.
        if pathlib.Path(normalized[0]).parent != pathlib.Path("."):
            return normalized, self.repo_root
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


def main() -> int:
    args = parse_args()
    repo_root = pathlib.Path(__file__).resolve().parent.parent.parent
    runner = Runner(
        repo_root=repo_root,
        serial=args.serial,
        qr_marker=args.qr_marker,
        helper_timeout=args.helper_timeout,
        logcat_tag=args.logcat_tag,
        wallet_verifier=args.wallet_verifier,
        require_wallet_test=args.require_wallet_test,
    )
    return runner.run(args.command)


if __name__ == "__main__":
    sys.exit(main())
