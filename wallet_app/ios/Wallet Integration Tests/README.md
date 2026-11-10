# Rust wallet integration tests on an iPhone

`testWalletCloseProximityDisclosureWithMacReader` runs the Rust wallet's `start`, `continue`, and
`accept_disclosure` APIs from the `Wallet Integration Tests` target in `wallet_app/ios/Runner.xcodeproj`.
It reuses PlatformSupport's minimal `Integration Tests Host App`, the real native BLE bridge, and
the Mac Swift reader. The Rust scenario is in `wallet_core/wallet/src/wallet/hardware_test/` and is
compiled only with `wallet/hardware_integration_test`. The Xcode integration-test bundle links the
dedicated `wallet_core/wallet/device_integration_tests` static-library crate, which includes both
`wallet` and `platform_support` without pulling in their development dependencies. The production
app does not link this test entry point.

The target depends on the existing `PlatformSupport` library and native host app. It keeps the
existing integration-test signing identifier/profile (`nl.ictu.edi.wallet.integration-tests`), so
no additional provisioning profile is needed. PlatformSupport's own test bundle builds and links
only the `platform_support` Rust crate. The two suites run separately and serially.

On a clean checkout, prepare the wallet project's generated Xcode metadata first. Xcode resolves
its Flutter package references even when the selected scheme only builds native targets:

```bash
cd wallet_app
flutter pub get --enforce-lockfile
(cd ios && bundle exec pod install --allow-root --deployment)
flutter build ios --config-only --no-codesign --debug --no-pub
cd ..
```

Use `fvm flutter` instead of `flutter` if using FVM. The Fastlane lane below performs this
preparation automatically. Building these tests does not run the Runner app or its Flutter engine.

Run from the repository root, with an unlocked iPhone connected and Bluetooth permissions granted:

```bash
scripts/close_proximity/run_disclosure_ios_test.py --require-wallet-test -- \
  xcodebuild test \
    -project wallet_app/ios/Runner.xcodeproj \
    -scheme 'Wallet Integration Tests' \
    -testPlan LocalRealDevice \
    -parallel-testing-enabled NO \
    -only-testing:'Wallet Integration Tests/CloseProximityDisclosureTests/testWalletCloseProximityDisclosureWithMacReader'
```

The wrapper builds the host Rust response verifier before launching XCTest. Pass
`--wallet-verifier <executable>` to reuse an already built verifier. `--require-wallet-test` fails
if no wallet response was verified, including when the wrong test plan skips the test.

Each run creates fresh test keys, a synthetic PID, and a registered, unlocked wallet backed by
in-memory SQLite. It generates a WRPAC, WRPRC, CRL, and signed WRPRC status list. Certificate and
authorization checks use the production code, with controlled CRL/status-list retrieval. A signed
test status list also covers the synthetic credential. A mock account-provider client signs the real
disclosure digest with the credential's private key and returns a signed instruction result.
This tests wallet signing orchestration, not the deployed Wallet Provider, HSM, PIN validation,
registration, issuance, or hardware-backed credential storage.

The scenario constructs reader authentication using the actual device engagement and the Mac
reader's known test ephemeral key. It passes the signed request through `device_request_hex`;
the Mac still performs BLE session establishment, encryption, transport, and response decryption.
This scenario does not test the Swift reader's separate WRPAC/WRPRC request-generation flags.
Only public certificates, the signed request, and synthetic test data leave the device.

The reader writes the received response and its session transcript to a temporary JSON artifact.
The host verifier checks issuer authentication, credential status, attribute digests, device authentication
bound to that transcript, and exactly the requested attribute and value. The wallet checks the proposal,
successful disclosure history, cleared session, and disconnect. The wrapper requires both XCTest
and reader verification to succeed; a missing artifact, failed signature, or timeout fails the run.
Tests must run serially because they share the device's native BLE bridge.

`testWalletJwtRegistrationCertificateWithMacReader` runs the same successful disclosure using a
compact JWT registration certificate in the `requestInfo.euWrprc` byte string. Both the JWT and CWT
scenarios enable strict WRPRC validation and check the proposal, signing, disclosure history, session
cleanup and cryptographically verified BLE response.

The `testWalletRegistrationCertificate*` tests exercise the EUDI registration-certificate rejection
paths through the same wallet and BLE transport. They cover missing/wrongly typed/malformed WRPRCs,
tampered signatures, expiry, untrusted signing chains, reader identity and subject-type mismatches,
revoked/suspended registrations, unavailable or untrusted status lists, out-of-bounds status indices,
and unauthorized document types, namespaces and attributes. A two-document request also checks that
an unauthorized second document rejects the entire request.

Each request is reader-signed after the invalid registration data is inserted. The wallet must return
the specific registration-certificate error, make no signing-instruction calls, record no disclosure,
clear the session and stop its update listener. The Mac reader must receive and decrypt an error-only
`DeviceResponse`: `InvalidRequest` for WRPRC parsing failures, `GeneralError` for the other cases.
The host verifier rejects missing responses, incorrect statuses and responses containing documents.
Status-list retrieval is controlled test infrastructure; the unavailable case returns a token-parsing
failure. These tests exercise this branch's EUDI WRPRC policy, not every ISO 18013-5 requirement.

All 19 tests run in the wallet project's `LocalRealDevice` plan and `test-ios-wallet-device` CI job.
To include them locally, select `Wallet Integration Tests/CloseProximityDisclosureTests` in the
command above, or select an individual `testWalletRegistrationCertificate*` method by its full name.

The same Rust scenarios can run without Bluetooth using a loopback transport:

```bash
cargo test --manifest-path wallet_core/Cargo.toml -p wallet \
  --features hardware_integration_test --test close_proximity_device_scenario
python3 -m unittest discover -s scripts/close_proximity -p 'test_*.py'
```

## CI and the complete device suite

```bash
IOS_DEVICE_UDID=<device-udid> bundle exec fastlane ios test_wallet_device
```

`deploy/gitlab/wallet-device-tests.yml` runs this lane on the Mac mini for default-branch and
merge-request pipelines. It shares `real-device-automation-tests` with the PlatformSupport and
other device jobs to prevent simultaneous access to the iPhone and Mac BLE reader.

The job publishes `fastlane/test_output/wallet-device.junit`, `wallet-device.log`, and
`WalletDevice.xcresult`. Coverage is enabled in the wallet test plan and retained in the result
bundle. PlatformSupport's existing job and Sonar coverage report remain scoped to its native tests.
