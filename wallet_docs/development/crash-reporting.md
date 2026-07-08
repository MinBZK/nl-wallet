# Crash Reporting

The mobile app uses Sentry for crash reporting, fatal exception capture, native
crash diagnostics, Rust panic diagnostics, and a small curated breadcrumb trail.
Selected non-production release builds can also opt into device logs and Sentry
Logs for troubleshooting. The setup is intentionally split across Flutter,
native Android/iOS, and Rust so each layer owns the failures it can diagnose best
while using the same release, environment, and privacy rules.

Sentry is enabled only when a non-empty `SENTRY_DSN` is provided. The configured
environment labels events but does not decide whether logs are enabled. Builds
without a DSN run without Sentry telemetry.

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
| `ALLOW_RELEASE_LOGS` | Enables device logs and Sentry Logs in profile/release builds. |
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

`ALLOW_RELEASE_LOGS` is a separate logging policy switch. Debug builds allow
logs by default. Profile and release builds allow logs only when
`ALLOW_RELEASE_LOGS=true`. CI defaults this flag to `false`; `ont` and `demo`
release builds opt in explicitly, while production builds keep it `false`.
Do not infer log allowance from `SENTRY_ENVIRONMENT`.

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

### Device Logs And Sentry Logs

Device logs and Sentry Logs are diagnostics for debug and explicitly opted-in
non-production release builds. They are not part of production crash-reporting
telemetry. We do not rely on log-message scrubbing for production privacy; the
production control is that logs stay disabled.

In this document, logs means diagnostic log records from app-owned logging
paths, such as Flutter/Fimber messages, Rust `tracing` records, and wallet-owned
native platform logs emitted through Android `android.util.Log` or iOS `NSLog`.
These are separate from Sentry crash/error events.

The shared log gate is:

- Debug builds: logs enabled.
- Profile/release builds with `ALLOW_RELEASE_LOGS=true`: logs enabled.
- Profile/release builds with `ALLOW_RELEASE_LOGS=false`: logs disabled.

Production builds must use the default `ALLOW_RELEASE_LOGS=false`. This means
Flutter Fimber logs, Rust tracing logs, Rust Sentry Logs, and wallet-owned
native platform-support logs are not emitted there. These records are also not
forwarded to Sentry Logs in production.

When logs are enabled:

- Flutter plants `DebugTree` and, if Sentry is enabled, `SentryLogTree`.
- Flutter `SentryLogTree` forwards Fimber logs to Sentry Logs.
- Rust tracing uses a `DEBUG` max level and forwards `debug`, `info`, `warn`,
  and `error` tracing records to Sentry Logs when Sentry is enabled.
- Rust ignores `trace` records for Sentry Logs.
- Android and iOS Rust tracing records are written to the platform log writers.
- Wallet-owned `platform_support` native code uses the same release-log gate.

When logs are disabled:

- Flutter does not plant `DebugTree` or `SentryLogTree`.
- Flutter `beforeSendLog` drops Sentry Logs defensively.
- Rust tracing uses `LevelFilter::OFF`.
- Rust Sentry `enable_logs` is disabled.
- Wallet-owned platform-support logs are suppressed.

The platform-support gate applies only to wallet-owned native logging paths. It
does not make a claim about third-party native libraries that write directly to
Android Logcat or iOS system logging. Those logs are outside this wrapper and
are not forwarded to Sentry Logs by our logging integration. Stripping such
third-party logs from Android release builds would require a separate
minification/ProGuard change and is not part of the Sentry logging design.

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

Production Sentry telemetry must not become a broad log sink. Crash/error events
are filtered before sending, breadcrumbs are curated at both insertion time and
send time, and device/Sentry Logs are disabled in production by keeping
`ALLOW_RELEASE_LOGS=false`.

Shared privacy rules:

- `sendDefaultPii` is disabled in Flutter, Android, iOS, and Rust.
- SDK-side user IP address and geo fields are removed before sending.
- Request and transaction data are removed from Rust events.
- Breadcrumb payload data is removed; only category and message code remain.
- Non-wallet breadcrumbs are dropped.
- Flutter exception values are removed before sending. Stacktraces remain.
- Rust exception values are removed for personal-data, unexpected, and
  uncategorized error events.
- Rust expected errors are not sent.
- Production device logs and Sentry Logs are not emitted.

The stable diagnostic signal is the event type, stacktrace, release,
environment, platform context, curated breadcrumb codes, and Rust error category
tag. Free-form user data and domain payloads are not part of crash reporting.
Sentry-side IP/geolocation enrichment is disabled in account/project settings
where possible, but the SDK cannot fully control metadata derived by Sentry from
transport-level IP information. Treat any residual IP/geolocation as
service-side metadata, not app-supplied event payload.

The current `ont` and `demo` Sentry projects are accepted as internal
non-production projects. Their service-level account setup, including data
storage region, must not be treated as production policy.

## Releases And Debug Symbols

Release names must align across Flutter, native Android/iOS, Rust, and uploaded
debug files. Symbol uploads are a release requirement whenever Sentry is enabled
for a publishable build.

Fastlane runs the Sentry Dart plugin for Flutter and app-native debug symbols.
The plugin runs only when `SENTRY_AUTH_TOKEN` is present and requires
`SENTRY_ORG`, `SENTRY_PROJECT`, and `SENTRY_URL`.

Fastlane computes `SENTRY_RELEASE` once per app build from the app identifier,
version, and build number unless it is supplied explicitly. The same release
value is passed to Flutter, Android `BuildConfig`, iOS build settings, Rust
compile-time environment, and all Sentry upload commands.

Android additionally uploads Rust debug files explicitly from the unstripped
Rust libraries copied into:

```text
wallet_app/android/app/src/main/jniLibs/**/libwallet_core.so
```

For Android profile/release builds, Cargo keeps Rust line-table debug
information in those CI-side libraries with
`CARGO_PROFILE_RELEASE_DEBUG=line-tables-only` and
`CARGO_PROFILE_RELEASE_STRIP=none`. Fastlane uploads those files with
`sentry-cli debug-files upload --include-sources --wait`. The Android packaging
pipeline then strips the native libraries in the distributed app artifact. This
provides Rust source-line symbolication in Sentry without shipping debug symbols
in the installed app.

iOS Rust code is statically linked into the final app binary. Rust symbolication
therefore uses the app dSYMs from:

```text
wallet_app/build/ios/archive/Runner.xcarchive
```

For iOS profile/release builds, the Xcode build also sets
`CARGO_PROFILE_RELEASE_DEBUG=line-tables-only` and
`CARGO_PROFILE_RELEASE_STRIP=none` before compiling Rust. The iOS archive is
retained as a CI artifact and uploaded through the Sentry Dart plugin path, so
the dSYMs carry the final app and statically linked Rust symbolication data.
