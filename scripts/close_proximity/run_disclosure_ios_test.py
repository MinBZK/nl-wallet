#!/usr/bin/env python3

import argparse
import json
import os
import pathlib
import re
import select
import shlex
import signal
import subprocess
import sys
import tempfile
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
        "--wallet-verifier",
        type=pathlib.Path,
        help="Prebuilt verify_close_proximity_response executable; otherwise build it before starting XCTest.",
    )
    parser.add_argument(
        "--require-wallet-test",
        action="store_true",
        help="Fail unless at least one Rust wallet disclosure is verified (detects skipped test selection).",
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
        self,
        repo_root: pathlib.Path,
        qr_marker: str,
        helper_timeout: int,
        wallet_verifier: Optional[pathlib.Path] = None,
        require_wallet_test: bool = False,
    ) -> None:
        self.repo_root = repo_root
        self.qr_pattern = re.compile(re.escape(qr_marker) + r"(\{[^\r\n]+\})")
        self.helper_timeout = helper_timeout
        self.test_process: Optional[subprocess.Popen[bytes]] = None
        self.helper_process: Optional[subprocess.Popen[bytes]] = None
        self.helper_exit_code = 0
        self.seen_payloads: set[str] = set()
        self._stopping = False
        self.wallet_verifier = wallet_verifier
        self.require_wallet_test = require_wallet_test
        self.verified_wallet_tests = 0
        self.artifacts = tempfile.TemporaryDirectory(prefix="wallet-disclosure-")
        self.pending_verification: Optional[tuple[pathlib.Path, pathlib.Path]] = None

    def _prepare_wallet_verifier(self) -> None:
        if self.wallet_verifier is not None:
            self.wallet_verifier = self.wallet_verifier.resolve(strict=True)
            return
        # Build before starting the device: compilation must not consume the BLE connection timeout.
        completed = subprocess.run(
            [
                "cargo", "build",
                "--manifest-path", str(self.repo_root / "wallet_core/Cargo.toml"),
                "-p", "wallet",
                "--features", "hardware_integration_test",
                "--example", "verify_close_proximity_response",
                "--message-format=json-render-diagnostics",
            ],
            cwd=self.repo_root,
            stdout=subprocess.PIPE,
            text=True,
            check=True,
        )
        for line in completed.stdout.splitlines():
            artifact = json.loads(line)
            if (
                artifact.get("reason") == "compiler-artifact"
                and artifact.get("target", {}).get("name") == "verify_close_proximity_response"
                and artifact.get("executable")
            ):
                self.wallet_verifier = pathlib.Path(artifact["executable"])
        if self.wallet_verifier is None:
            raise RuntimeError("Cargo did not produce the wallet response verifier")

    def run(self, command: list[str]) -> int:
        self._prepare_wallet_verifier()
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
                self._poll_helper()
                self._maybe_start_helper(buffer)
                # A previous reader may still be sending BLE end when the next test emits its marker.
                # Keep launching pending readers even while XCTest is silent waiting for Bluetooth.
                if not select.select([master_fd], [], [], 0.1)[0]:
                    continue
                try:
                    chunk = os.read(master_fd, 4096)
                except OSError:
                    break

                if not chunk:
                    break

                text = chunk.decode("utf-8", errors="replace")
                sys.stdout.write(text)
                sys.stdout.flush()

                # Signed requests and their test trust anchors exceed the old 8 KiB log window.
                buffer = (buffer + text)[-1_048_576:]
        finally:
            os.close(master_fd)

        test_exit_code = self.test_process.wait() if self.test_process else 1
        helper_exit_code = self._finish_helper()
        self.artifacts.cleanup()

        if test_exit_code != 0:
            return test_exit_code
        if helper_exit_code != 0:
            return helper_exit_code
        if self.require_wallet_test and self.verified_wallet_tests == 0:
            print("No Rust wallet disclosure was verified; check the test plan and selection.", file=sys.stderr)
            return 1
        return 0

    def _maybe_start_helper(self, buffer: str) -> None:
        payload_text = next(
            (match.group(1) for match in self.qr_pattern.finditer(buffer)
             if match.group(1) not in self.seen_payloads),
            None,
        )
        if payload_text is None:
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

        wrpac_crt_file = payload.get("wrpac_crt_file")
        if isinstance(wrpac_crt_file, str) and wrpac_crt_file:
            helper_command.extend(["--wrpac-crt-file", wrpac_crt_file])

        wrpac_key_file = payload.get("wrpac_key_file")
        if isinstance(wrpac_key_file, str) and wrpac_key_file:
            helper_command.extend(["--wrpac-key-file", wrpac_key_file])

        wrprc_file = payload.get("wrprc_file")
        if isinstance(wrprc_file, str) and wrprc_file:
            helper_command.extend(["--wrprc-file", wrprc_file])

        request_doc_type = payload.get("request_doc_type")
        if isinstance(request_doc_type, str) and request_doc_type:
            helper_command.extend(["--request-doc-type", request_doc_type])

        request_attributes = payload.get("request_attributes")
        if isinstance(request_attributes, list):
            for attribute in request_attributes:
                if isinstance(attribute, str) and attribute:
                    helper_command.extend(["--request-attribute", attribute])

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
        verification = payload.get("wallet_verification")
        if verification is not None:
            if not isinstance(verification, dict) or self.wallet_verifier is None:
                raise RuntimeError("Wallet disclosure requires verification metadata and a host verifier")
            directory = pathlib.Path(self.artifacts.name) / str(len(self.seen_payloads))
            directory.mkdir()
            expected = directory / "expected.json"
            received = directory / "received.json"
            expected.write_text(json.dumps(verification))
            self.pending_verification = (expected, received)
            helper_command.extend(["--device-response-file", str(received)])
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
        self._record_helper_result(helper_exit_code)
        self.helper_process = None

    def _record_helper_result(self, helper_exit_code: int) -> None:
        if self.pending_verification is not None:
            expected, received = self.pending_verification
            self.pending_verification = None
            if helper_exit_code == 0:
                if not received.is_file():
                    print("BLE reader did not write a response artifact", file=sys.stderr)
                    helper_exit_code = 1
                else:
                    try:
                        verification = subprocess.run(
                            [str(self.wallet_verifier), str(expected), str(received)],
                            cwd=self.repo_root,
                            timeout=30,
                            check=False,
                        )
                        helper_exit_code = verification.returncode
                        if helper_exit_code == 0:
                            self.verified_wallet_tests += 1
                    except (OSError, subprocess.TimeoutExpired) as error:
                        print(f"Wallet response verification failed: {error}", file=sys.stderr)
                        helper_exit_code = 1
        if helper_exit_code != 0 and self.helper_exit_code == 0:
            self.helper_exit_code = helper_exit_code

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

        self._record_helper_result(helper_exit_code)
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
        wallet_verifier=args.wallet_verifier,
        require_wallet_test=args.require_wallet_test,
    )
    return runner.run(args.command)


if __name__ == "__main__":
    sys.exit(main())
