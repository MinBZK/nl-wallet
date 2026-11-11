#!/usr/bin/env python3

import argparse
import json
import os
import pathlib
import select
import signal
import subprocess
import sys
from typing import Optional

from disclosure_test_reader import Reader


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


class Runner(Reader):
    def __init__(self, *args, **kwargs) -> None:
        super().__init__(*args, **kwargs)
        self.test_process: Optional[subprocess.Popen[bytes]] = None
        self._stopping = False

    def run(self, command: list[str]) -> int:
        try:
            return self._run(command)
        except BaseException:
            self._stopping = True
            try:
                if self.test_process is not None:
                    self._stop_process_group(self.test_process)
            finally:
                if self.helper_process is not None:
                    self._stop_process_group(self.helper_process)
            raise
        finally:
            self.artifacts.cleanup()

    def _run(self, command: list[str]) -> int:
        self._prepare_wallet_verifier()
        command = self._with_auto_destination(command)
        master_fd, slave_fd = os.openpty()
        try:
            try:
                self.test_process = subprocess.Popen(
                    command,
                    cwd=self.repo_root,
                    stdin=slave_fd,
                    stdout=slave_fd,
                    stderr=slave_fd,
                    start_new_session=True,
                )
            finally:
                os.close(slave_fd)

            signal.signal(signal.SIGINT, self._handle_signal)
            signal.signal(signal.SIGTERM, self._handle_signal)

            buffer = ""
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

        if test_exit_code != 0:
            return test_exit_code
        if helper_exit_code != 0:
            return helper_exit_code
        if self.require_wallet_test and self.verified_wallet_tests == 0:
            print("No Rust wallet disclosure was verified; check the test plan and selection.", file=sys.stderr)
            return 1
        return 0

    def _handle_signal(self, signum: int, _frame) -> None:
        if self._stopping:
            return

        self._stopping = True
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
