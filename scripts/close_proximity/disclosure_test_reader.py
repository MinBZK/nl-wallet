"""Mac BLE reader and Rust response verification shared by both device runners."""

import json
import os
import pathlib
import re
import shlex
import signal
import subprocess
import sys
import tempfile
from typing import Optional


class Reader:
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
        self.helper_process: Optional[subprocess.Popen[bytes]] = None
        self.helper_exit_code = 0
        self.seen_payloads: set[str] = set()
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
                "cargo", "build", "--locked",
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

    def _maybe_start_helper(self, buffer: str) -> None:
        # Even an exited helper owns its result until _poll_helper() records it.
        if self.helper_process is not None:
            return

        for match in self.qr_pattern.finditer(buffer):
            payload_text = match.group(1)
            if payload_text in self.seen_payloads:
                continue
            self.seen_payloads.add(payload_text)

            try:
                payload = json.loads(payload_text)
            except json.JSONDecodeError:
                continue

            qr_code = payload.get("qr")
            if not isinstance(qr_code, str) or not qr_code:
                continue
            break
        else:
            return

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
            start_new_session=True,
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
            helper_exit_code = self._stop_process_group(self.helper_process)

        self._record_helper_result(helper_exit_code)
        self.helper_process = None
        return self.helper_exit_code

    @staticmethod
    def _stop_process_group(process: subprocess.Popen[bytes]) -> int:
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            pass
        # A shell may exit before its descendants, including xcodebuild, have stopped.
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        return process.wait()
