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
from typing import Optional


# Example usage from the nl_wallet root:
# scripts/close_proximity/run_disclosure_ios_test.py -- xcodebuild test -project wallet_core/wallet/platform_support/ios/PlatformSupport.xcodeproj -scheme PlatformSupport -testPlan LocalRealDevice -only-testing:'Integration Tests/CloseProximityDisclosureTests/testCloseProximityDisclosureFullFlowWithMacReader'
def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run an iOS close-proximity test command, watch for the close-proximity host-helper marker "
            "in the output, and launch the host Mac BLE helper automatically. If the command is "
            "xcodebuild and no -destination is provided, the script auto-selects the single "
            "connected physical iOS device."
        )
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
        "command",
        nargs=argparse.REMAINDER,
        help="Command to execute. Pass it after --, for example: -- xcodebuild test ...",
    )
    args = parser.parse_args()

    if args.command and args.command[0] == "--":
        args.command = args.command[1:]

    if not args.command:
        parser.error("Missing command to run. Pass it after --.")

    return args


class Runner:
    def __init__(
        self, repo_root: pathlib.Path, qr_marker: str, helper_timeout: int
    ) -> None:
        self.repo_root = repo_root
        self.qr_pattern = re.compile(re.escape(qr_marker) + r"(\{[^\r\n]+\})")
        self.helper_timeout = helper_timeout
        self.test_process: Optional[subprocess.Popen[bytes]] = None
        self.helper_process: Optional[subprocess.Popen[bytes]] = None
        self.helper_exit_code = 0
        self.seen_payloads: set[str] = set()
        self._stopping = False

    def run(self, command: list[str]) -> int:
        command = self._with_auto_destination(command)
        master_fd, slave_fd = os.openpty()
        try:
            self.test_process = subprocess.Popen(
                command,
                cwd=self.repo_root,
                stdin=slave_fd,
                stdout=slave_fd,
                stderr=slave_fd,
            )
        finally:
            os.close(slave_fd)

        signal.signal(signal.SIGINT, self._handle_signal)
        signal.signal(signal.SIGTERM, self._handle_signal)

        buffer = ""

        try:
            while True:
                try:
                    chunk = os.read(master_fd, 4096)
                except OSError:
                    break

                if not chunk:
                    break

                text = chunk.decode("utf-8", errors="replace")
                sys.stdout.write(text)
                sys.stdout.flush()

                buffer = (buffer + text)[-8192:]
                self._maybe_start_helper(buffer)

                self._poll_helper()
        finally:
            os.close(master_fd)

        test_exit_code = self.test_process.wait() if self.test_process else 1
        helper_exit_code = self._finish_helper()

        if test_exit_code != 0:
            return test_exit_code
        if helper_exit_code != 0:
            return helper_exit_code
        return 0

    def _maybe_start_helper(self, buffer: str) -> None:
        match = self.qr_pattern.search(buffer)
        if not match:
            return

        payload_text = match.group(1)
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
        if (
            isinstance(expected_device_response_hex, str)
            and expected_device_response_hex
        ):
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
        self.helper_process = subprocess.Popen(
            helper_command,
            cwd=self.repo_root,
            stdout=sys.stderr.buffer,
            stderr=sys.stderr.buffer,
        )

    def _poll_helper(self) -> None:
        if not self.helper_process:
            return
        helper_exit_code = self.helper_process.poll()
        if helper_exit_code is None:
            return
        print(
            f"\n[auto-close-proximity] Helper exited with code {helper_exit_code}\n",
            file=sys.stderr,
            flush=True,
        )
        if helper_exit_code != 0 and self.helper_exit_code == 0:
            self.helper_exit_code = helper_exit_code
        self.helper_process = None

    def _finish_helper(self) -> int:
        if not self.helper_process:
            return self.helper_exit_code

        try:
            helper_exit_code = self.helper_process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.helper_process.terminate()
            try:
                helper_exit_code = self.helper_process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.helper_process.kill()
                helper_exit_code = self.helper_process.wait()

        if helper_exit_code != 0 and self.helper_exit_code == 0:
            self.helper_exit_code = helper_exit_code
        self.helper_process = None
        return self.helper_exit_code

    def _handle_signal(self, signum: int, _frame) -> None:
        if self._stopping:
            return

        self._stopping = True
        if self.test_process and self.test_process.poll() is None:
            self.test_process.terminate()
        if self.helper_process and self.helper_process.poll() is None:
            self.helper_process.terminate()
        raise SystemExit(128 + signum)

    def _with_auto_destination(self, command: list[str]) -> list[str]:
        if not command:
            return command
        if pathlib.Path(command[0]).name != "xcodebuild":
            return command
        if "-destination" in command:
            return command

        destination = self._detect_destination()
        if not destination:
            return command

        patched_command = command.copy()
        insert_index = 1
        patched_command[insert_index:insert_index] = [
            "-destination",
            f"id={destination}",
        ]
        print(
            f"[auto-close-proximity] Using connected device destination id={destination}",
            file=sys.stderr,
            flush=True,
        )
        return patched_command

    def _detect_destination(self) -> Optional[str]:
        try:
            completed = subprocess.run(
                ["xcrun", "xcdevice", "list"],
                cwd=self.repo_root,
                capture_output=True,
                text=True,
                check=True,
            )
        except subprocess.CalledProcessError as error:
            print(
                f"[auto-close-proximity] Failed to detect connected devices via xcdevice: {error}",
                file=sys.stderr,
                flush=True,
            )
            return None

        devices = self._parse_xcdevice_json(completed.stdout)
        physical_ios_devices = [
            device
            for device in devices
            if device.get("available") is True
            and device.get("simulator") is False
            and device.get("platform") == "com.apple.platform.iphoneos"
        ]

        if not physical_ios_devices:
            print(
                "[auto-close-proximity] No connected physical iOS device found; leaving the xcodebuild command unchanged.",
                file=sys.stderr,
                flush=True,
            )
            return None

        if len(physical_ios_devices) > 1:
            rendered_devices = ", ".join(
                f"{device.get('name', '<unknown>')} ({device.get('identifier', '<missing>')})"
                for device in physical_ios_devices
            )
            raise SystemExit(
                "[auto-close-proximity] Multiple connected iOS devices found. "
                f"Pass -destination manually. Devices: {rendered_devices}"
            )

        return physical_ios_devices[0].get("identifier")

    @staticmethod
    def _parse_xcdevice_json(output: str) -> list[dict]:
        json_start = output.rfind("\n[")
        if json_start >= 0:
            candidate = output[json_start + 1 :]
        else:
            json_start = output.find("[")
            if json_start < 0:
                raise SystemExit(
                    "[auto-close-proximity] Could not find xcdevice JSON output."
                )
            candidate = output[json_start:]

        try:
            parsed = json.loads(candidate)
        except json.JSONDecodeError as error:
            raise SystemExit(
                f"[auto-close-proximity] Failed to parse xcdevice output: {error}"
            ) from error

        if not isinstance(parsed, list):
            raise SystemExit(
                "[auto-close-proximity] Unexpected xcdevice output shape; expected a JSON array."
            )
        return parsed


def main() -> int:
    args = parse_args()
    repo_root = pathlib.Path(__file__).resolve().parent.parent.parent
    runner = Runner(
        repo_root=repo_root,
        qr_marker=args.qr_marker,
        helper_timeout=args.helper_timeout,
    )
    return runner.run(args.command)


if __name__ == "__main__":
    sys.exit(main())
