# Rust wallet integration tests on iOS and Android

Both platforms run the same 19 Rust wallet scenarios: successful CWT and JWT registration-certificate
flows and 17 registration-certificate rejection cases. They exercise `start`, `continue`, and
`accept_disclosure` through the real native BLE bridge, with the Mac Swift helper as the reader.

The native harnesses belong to the wallet app:

- iOS: `Wallet Integration Tests` in `wallet_app/ios/Runner.xcodeproj`, using PlatformSupport's
  minimal `Integration Tests Host App`.
- Android: `wallet_app/android/wallet-integration-tests`, a native test host with Kotlin
  instrumentation tests and a dependency on PlatformSupport's Android library.

The scenarios live in `wallet_core/wallet/src/wallet/hardware_test/`, behind
`wallet/hardware_integration_test`. This `device_integration_tests` crate exposes their C entry
points as a static library for iOS and a shared library for Android. It includes `wallet` and
`platform_support` without their development dependencies. Neither production app links these
entry points. PlatformSupport's own suites retain the lower-level bridge and lifecycle tests.

## Scenarios and verification

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
successful disclosure history, cleared session, and disconnect. The wrappers require both the device tests
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

## Running on a device

Both complete device suites need a Mac with Bluetooth and Swift, a connected and unlocked physical
phone with Bluetooth enabled, and Bluetooth permission for the terminal running the reader.
Android also needs USB debugging authorized. Run the suites serially: they share the native bridge
and the Mac reader.

With `--require-wallet-test`, the wrappers build the host Rust response verifier before starting the device tests. Pass
`--wallet-verifier <executable>` to reuse an already built verifier. `--require-wallet-test` fails
if no wallet response was verified, including when the test selection skips the wallet suite.

### iOS

The iOS target depends on the existing `PlatformSupport` library and native host app. It keeps the
existing integration-test signing identifier/profile (`nl.ictu.edi.wallet.integration-tests`), so
no additional provisioning profile is needed. PlatformSupport's own test bundle builds and links
only the `platform_support` Rust crate. The two suites run separately and serially.

On a clean checkout, prepare Flutter's iOS artifacts and the wallet project's generated Xcode metadata first.
Xcode resolves its Flutter package references even when the selected scheme only builds native targets:

```bash
cd wallet_app
flutter pub get --enforce-lockfile
flutter precache --ios
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
    -only-testing:'Wallet Integration Tests/CloseProximityDisclosureTests'
```

Append `/testWalletCloseProximityDisclosureWithMacReader` (or another method name) to the
`-only-testing` selection above to run an individual case.

### Android

Install the Android SDK, NDK version from the module's `build.gradle.kts`, `cargo-ndk`, and the Rust
Android target for your device. Set `ANDROID_HOME`, `ANDROID_NDK_HOME`, and `ANDROID_NDK_TARGETS`
accordingly. For the arm64 CI device, use `ANDROID_NDK_TARGETS=arm64-v8a` and the
`aarch64-linux-android` Rust target.

Run from the repository root:

```bash
ANDROID_NDK_TARGETS=arm64-v8a scripts/close_proximity/run_disclosure_android_test.py \
  --require-wallet-test -- \
  wallet_app/android/gradlew -p wallet_app/android/wallet-integration-tests \
  :connectedDebugAndroidTest
```

Use `--serial <adb-serial>` before `--` when multiple devices are connected. The standalone Gradle
entry point avoids configuring the Flutter app. The module is also included in the wallet Android
project for IDE use. It is a debug-only test host and does not start Flutter.

The host's manifest selects `libdevice_integration_tests.so` for the UniFFI bridge, and packaging
excludes `libplatform_support.so`. This ensures that bridge initialization and wallet scenarios
use the same Rust singleton. Kotlin calls the same C entry points as Swift via JNA, on a worker
thread with a larger stack. It copies and frees Rust errors and reports them as JUnit failures.

Signed requests and verification fixtures exceed logcat's per-message limit. Kotlin emits numbered
base64 chunks; the Android wrapper reconstructs the marker before launching the shared Mac reader
and verifier. Missing or malformed chunks fail the run.

To select one scenario, add:

```text
-Pandroid.testInstrumentationRunnerArguments.class=nl.rijksoverheid.edi.wallet.integration_tests.CloseProximityDisclosureTests#testWalletRegistrationCertificateMissing
```

Two additional adapter checks verify bridge initialization and Rust error propagation without a
Mac reader. They can run directly on an Android device from Linux as well:

```bash
ANDROID_NDK_TARGETS=arm64-v8a wallet_app/android/gradlew \
  -p wallet_app/android/wallet-integration-tests :connectedDebugAndroidTest \
  -Pandroid.testInstrumentationRunnerArguments.class=nl.rijksoverheid.edi.wallet.integration_tests.WalletTestLibraryTest
```

These checks do not exercise a disclosure exchange and do not replace the 19 reader-backed cases.

## Host-only checks

The same Rust scenarios can run without Bluetooth using a loopback transport:

```bash
cargo test --manifest-path wallet_core/Cargo.toml -p wallet \
  --features hardware_integration_test --test close_proximity_device_scenario
python3 -m unittest discover -s scripts/close_proximity -p 'test_*.py'
```

## CI and the complete device suites

```bash
IOS_DEVICE_UDID=<device-udid> bundle exec fastlane ios test_wallet_device
ANDROID_SERIAL=<adb-serial> bundle exec fastlane android test_wallet_device
```

`deploy/gitlab/wallet-device-tests.yml` runs `test-ios-wallet-device` and `test-android-wallet-device`
on the Mac mini in regular default-branch pipelines. On merge requests, a change to
the shared Rust wallet scenarios selects both jobs. A change to one native wallet
test harness selects that platform's job. Both share
`real-device-automation-tests` with the PlatformSupport and other device jobs to prevent
simultaneous access to the phones and Mac BLE reader.

The iOS job publishes `fastlane/test_output/wallet-device.junit`, `wallet-device.log`, and
`WalletDevice.xcresult`. Coverage is enabled in its test plan and retained in the result bundle.
The Android job publishes JUnit XML and Gradle reports from
`wallet_app/android/wallet-integration-tests/build/`. PlatformSupport's existing jobs and Sonar
coverage reports remain scoped to its native tests.
