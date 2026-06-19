# Crash Reporting

The mobile app uses Sentry for crash reporting, fatal exception capture, native
crash diagnostics, Rust panic diagnostics, and a small curated breadcrumb trail.
The setup is intentionally split across Flutter, native Android/iOS, and Rust so
each layer owns the failures it can diagnose best while using the same release,
environment, and privacy rules.

Sentry is enabled only when a non-empty `SENTRY_DSN` is provided. The configured
environment labels events but does not change runtime behavior. Builds without a
DSN run without Sentry telemetry.

## Runtime Ownership

Each runtime owns one failure domain:

- Flutter owns uncaught Dart exceptions and explicit Dart-side error events.
- Android and iOS own native crashes, Android ANRs, iOS app hangs, watchdog
  terminations, tombstones, and native crash artifacts.
- Rust owns Rust panics and categorized Rust error events emitted through
  `#[sentry_capture_error]`.

Native Sentry startup lives in the app shells:

- Android initializes Sentry from `WalletApplication`.
- iOS initializes Sentry from `AppDelegate`.
- `platform_support` stays limited to bridge/runtime support and does not own
  crash-reporting policy.

Flutter initializes Sentry around the app runner and disables implicit native
SDK ownership:

- `autoInitializeNativeSdk = false`
- `enableNativeCrashHandling = false`

That keeps native crash capture under the explicit Android and iOS SDK
configuration while still allowing Flutter to capture Dart events.

Rust initializes Sentry during `WalletCore.init()`. The panic integration is
installed before the wallet panic policy so the Rust SDK can capture and flush a
panic before the process exits.

## Configuration

The same build inputs feed Flutter, native, and Rust:

| Variable | Purpose |
| --- | --- |
| `SENTRY_DSN` | Enables Sentry and selects the Sentry project endpoint. |
| `SENTRY_ENVIRONMENT` | Labels events with the build/runtime environment. |
| `SENTRY_RELEASE` | Aligns Flutter, native, Rust, and uploaded debug files to one release. |
| `SENTRY_AUTH_TOKEN` | Enables release debug-symbol upload in CI/Fastlane. |
| `SENTRY_ORG` | Sentry organization used by debug-symbol upload. |
| `SENTRY_PROJECT` | Sentry project used by debug-symbol upload. |
| `SENTRY_URL` | Sentry base URL used by debug-symbol upload. |

Fastlane computes `SENTRY_RELEASE` from the app identifier, version, and build
number when no release is supplied explicitly. The computed release is passed to
Flutter as a Dart define, into Android `BuildConfig`, into iOS build settings,
and into the Rust compile environment. Rust prefers `SENTRY_RELEASE` and only
falls back to its crate release name when the app release is absent.

CI loads Sentry upload credentials from the environment-specific Sentry secret.
If `SENTRY_AUTH_TOKEN` is present, the release build must upload debug symbols
before publishing.

## Event Sources

Fatal Dart failures are captured through one fatal path. The app installs a
`PlatformDispatcher.onError` integration inside Sentry, captures one fatal
Sentry event with the original Dart stacktrace, gives the transport a short time
to drain, closes Sentry, and exits the process. The same fatal handler is
installed without Sentry when no DSN is configured, so fatal Dart behavior is
consistent.

Explicit Dart captures remain explicit events. Repository and bridge code
capture handled exceptions when the event is useful for diagnostics.

Android native Sentry captures native crashes, ANRs, NDK/tombstone artifacts,
and raw tombstone data where supported by the SDK and platform. Android 11 and
newer report ANRs through `ApplicationExitInfo`; a recoverable stall that does
not terminate the process is not an ANR event.

iOS native Sentry captures native crashes, app hangs, watchdog terminations, and
the app dSYM-based symbolication data for the final app binary.

Rust Sentry captures:

- Rust panics through the Rust panic integration.
- Categorized Rust errors emitted through `#[sentry_capture_error]`.
- A Rust breadcrumb for non-expected categorized errors.

Rust expected errors are dropped. Rust personal-data and uncategorized events are
sent only after sensitive message values are removed. Rust unexpected events are
sent scrubbed. Rust critical events keep their exception messages, but still use
the shared request, user, and breadcrumb scrubbing rules.

### Annotated Rust Errors

Rust non-panic diagnostics are sent through functions and implementations
annotated with `#[sentry_capture_error]`. The annotation captures returned
errors from the annotated scope and sends them through the Rust
`ErrorCategory` policy. The error type must derive or implement
`ErrorCategory`, and every variant or field must resolve to one of the Sentry
categories.

Rust error categories map to Sentry behavior as follows:

| Category | Sentry behavior |
| --- | --- |
| `expected` | Drop the event. |
| `critical` | Send the event with exception messages intact. |
| `pd` | Send the event with sensitive exception messages removed. |
| `unexpected` | Log locally and send the event with sensitive exception messages removed. |
| `defer` | Resolve the category from the wrapped or nested error. |

The annotation is transitive across called functions: an unhandled categorized
error returned through an annotated function is classified at that boundary. For
non-expected categories, Rust also emits a curated `wallet.native` breadcrumb
with the message code `rust.error.<category>`.

Adding or changing a `#[sentry_capture_error]` boundary is a privacy-sensitive
change. The matching error types need category annotations before the boundary
is useful, and any variant whose display text can contain domain data, URLs,
tokens, identifiers, or user-controlled text belongs in `#[category(pd)]` or a
deferred wrapper that resolves to `pd`.

Native crash artifacts belong only to native failures. Dart exceptions and Rust
categorized errors do not need separate minidump or coredump artifacts.

## Breadcrumbs

Breadcrumbs provide recent failure context. They are not logs, route traces, or
analytics. The app keeps only a small wallet-owned breadcrumb vocabulary that is
safe to copy between Flutter, native, and Rust Sentry scopes.

Allowed breadcrumb shape:

| Field | Allowed value |
| --- | --- |
| `category` | `wallet.flow` or `wallet.native` |
| `message` | Lowercase dotted code, for example `issuance.start` or `rust.error.critical`. |
| `data` | Removed before the breadcrumb is stored or sent. |
| `level` | Forced to `info`. |
| `type` | Forced to `default`. |

The message format is:

```text
^[a-z0-9_]+(\.[a-z0-9_]+)*$
```

Examples:

- `setup.start`
- `setup.fail.config_load`
- `issuance.start`
- `disclosure.blocked.finish_active_disclosure_session`
- `native.bridge_missing.method_channel`
- `rust.error.critical`

Breadcrumbs must not contain route names, organization names, document types,
URLs, user identifiers, free text, PIN state, payload fragments, or arbitrary
metadata. Add a breadcrumb only when it materially improves failure diagnosis.

Flutter is the cross-layer breadcrumb bridge:

- Flutter emits curated `wallet.flow` breadcrumbs around major wallet-core
  calls and failure boundaries.
- Rust emits curated `wallet.native` breadcrumbs for panic and categorized-error
  context.
- Rust forwards its breadcrumb message to Flutter after wallet-core
  initialization.
- Native Sentry scopes apply the same category, message, and payload filter so
  native crash, ANR, app-hang, and watchdog events retain the same recent
  curated context.

All layers use the same maximum breadcrumb count of 25.

## Privacy And PII Handling

Sentry must never become a broad log sink. Events are filtered before sending,
and breadcrumbs are curated at both insertion time and send time.

Shared privacy rules:

- `sendDefaultPii` is disabled in Flutter, Android, iOS, and Rust.
- User IP address and geo fields are removed before sending.
- Request and transaction data are removed from Rust events.
- Breadcrumb payload data is removed; only category and message code remain.
- Non-wallet breadcrumbs are dropped.
- Flutter exception values are removed before sending. Stacktraces remain.
- Rust exception values are removed for personal-data, unexpected, and
  uncategorized error events.
- Rust expected errors are not sent.

The stable diagnostic signal is the event type, stacktrace, release,
environment, platform context, curated breadcrumb codes, and Rust error category
tag. Free-form user data and domain payloads are not part of crash reporting.

## Releases And Debug Symbols

Release names must align across Flutter, native Android/iOS, Rust, and uploaded
debug files. Symbol uploads are a release requirement whenever Sentry is enabled
for a publishable build.

Fastlane runs the Sentry Dart plugin for Flutter and app-native debug symbols.
The plugin runs only when `SENTRY_AUTH_TOKEN` is present and requires
`SENTRY_ORG`, `SENTRY_PROJECT`, and `SENTRY_URL`.

Android additionally uploads Rust debug symbols from:

```text
wallet_app/android/app/src/main/jniLibs/**/libwallet_core.so
```

The Android build keeps line-table debug information for the CI-side Rust symbol
artifact and lets the Android packaging pipeline strip the distributed app
artifact. This provides Rust source-line symbolication in Sentry without
shipping debug symbols in the installed app.

iOS Rust code is statically linked into the final app binary. Rust symbolication
therefore uses the app dSYMs from:

```text
wallet_app/build/ios/archive/Runner.xcarchive
```

The iOS archive is retained as a CI artifact and uploaded through the normal
Sentry symbol upload path.

## Operational Checks

When changing Sentry setup, validate a Sentry-enabled non-production build with
the same release name across layers:

- A fatal Dart exception produces one fatal Dart event with a Dart stacktrace.
- A Rust panic produces one Rust panic event with an aligned release.
- A categorized Rust non-expected error produces one categorized Rust event.
- An Android native crash produces a native event with native symbolication.
- An Android ANR produces an event after system termination and relaunch on
  Android 11 or newer.
- An iOS native crash produces a native event with dSYM symbolication.
- An iOS app hang or watchdog termination produces a native event.
- Relevant events contain only curated `wallet.flow` and `wallet.native`
  breadcrumbs.
- Events do not include user geo, IP address, request data, route names, URLs,
  identifiers, or arbitrary breadcrumb payload data.
- Sentry debug files include Flutter/app-native symbols, Android
  `libwallet_core.so`, and iOS app dSYMs for the same release.
