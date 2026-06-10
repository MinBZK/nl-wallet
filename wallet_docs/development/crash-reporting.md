# PVW-5930 Design: Cross-Layer Sentry Crash, Exception, and Breadcrumb Architecture

Date: 2026-06-10 Ticket: `PVW-5930` Status: Draft design document for review

## Background

This design defines a clear Sentry ownership model across Flutter, native, and
Rust so failures from each layer produce one diagnosable artifact with aligned
release and symbolication data. It also proposes a small curated breadcrumb
model so crash reports include only the recent context needed to explain what
the app was doing before failure.

The current ONT baseline shows that Rust panic and categorized-error capture
already work well, and Android native crash and ANR capture are largely working,
while Flutter uncaught exceptions and iOS native crash/app-hang coverage are
still incomplete. The main architectural change is therefore to make ownership
and release alignment explicit, and to retain a minimal set of high-value
breadcrumbs across layers.

## Scope

- Mobile app runtime only
- Sentry-related crash, exception, stacktrace, symbolication, breadcrumb, and
  targeted-event behavior
- Existing Sentry-compatible toolchain only

## Out of scope

- Redesigning PII policy beyond the breadcrumb naming constraints needed here

Environment note:

- For now, runtime Sentry behavior stays the same across all environments where
  Sentry is configured.
- The only environment-specific switch is whether Sentry is enabled at all,
  which remains controlled by whether a DSN is provided for that build or run.
- `SENTRY_ENVIRONMENT` remains an event-labeling input, not a separate behavior
  switch.

## Target Design

### 1. Ownership and Bootstrap

Each failure domain should have one clear owner:

- Flutter SDK owns uncaught Dart exceptions and explicit Dart-side error events.
- Native iOS and Android SDKs own native crashes, ANRs, app hangs, and native
  crash artifacts.
- Rust SDK owns Rust panics and Rust categorized error events raised by
  `#[sentry_capture_error]`.

Target startup order:

1. Native SDK init in app-owned startup
    - Android: app startup path in the application shell, alongside but not
      inside `PlatformSupportInitializer`
    - iOS: `AppDelegate.didFinishLaunchingWithOptions`
2. Flutter Sentry init
3. Dart uncaught-error wiring
4. `WalletCore.init()` / `postInit()`
5. Breadcrumb bridge registration

Flutter should be initialized with:

- `autoInitializeNativeSdk = false`
- `enableNativeCrashHandling = false`

That keeps native crash handling owned by the native SDKs we initialize
ourselves and avoids implicit or duplicate ownership.

Ownership split:

- App-owned Android and iOS startup code owns native Sentry initialization.
- `platform_support` remains limited to wallet-core bridge/runtime services.
- If startup config assembly needs shared code, use a thin helper called from
  the app shells rather than moving Sentry ownership into `platform_support`.

### 2. Config, Release, and Symbolication

All layers should use the same source inputs:

- `SENTRY_DSN`
- `SENTRY_ENVIRONMENT`
- `SENTRY_RELEASE`

Desired flow:

- Flutter continues receiving these via Dart defines.
- Android and iOS derive native init inputs from the same build inputs and
  initialize native Sentry explicitly.
- Rust receives the same values at compile time and prefers `SENTRY_RELEASE`
  over `release_name!()`.

Release design requirements:

- Environment and release are aligned across Flutter, native, and Rust, and
  existing app/native symbol upload through `sentry_dart_plugin` is treated as
  required for Sentry-enabled release builds.
- iOS Rust symbolication rides on the final app dSYM because `libwallet_core.a`
  is statically linked into the app binary.
- Android Rust debug information is retained and uploaded explicitly from the
  packaged `libwallet_core.so` files as part of the same release process.

### 3. Exception, Panic, and Native Crash Behavior

- Flutter keeps one uncaught-exception capture path only.
- Flutter preserves crash-after-capture behavior for truly fatal uncaught
  failures.
- Rust keeps the current panic capture and `#[sentry_capture_error]` model.
- Native iOS / Android add explicit SDK initialization and enable native crash
  capture.
- Android enables ANR capture.
- iOS enables app-hang / watchdog-relevant tracking.

Native crash artifacts belong only to native failures. Dart exceptions and Rust
categorized errors do not need separate coredumps or minidumps.

### 4. Curated Breadcrumbs

Breadcrumbs are curated failure context. The app should emit only a small set of
breadcrumbs for major flow transitions and failure or blocked states.

Use a simple convention:

- `category`: `wallet.flow` or `wallet.native`
- `message`: one dotted code such as `issuance.start`,
  `disclosure.blocked.finish_active_disclosure_session`,
  `setup.fail.config_load`, or `native.bridge_missing.method_channel`

Flutter and Rust breadcrumbs should be forwarded into native scope so native
crash, ANR, and app-hang events include the same recent context. The initial
producer set should stay small: major flow boundaries in Flutter, panic and
categorized-error context in Rust, and lifecycle, init, and bridge failures in
native. Do not include route names, free text, identifiers, or arbitrary payload
fields. Add a breadcrumb only when it materially improves failure diagnosis.

## Current Architecture

### Flutter

- `wallet_app/lib/main.dart` initializes `WalletCore` before
  `SentryFlutter.init()`.
- `PlatformDispatcher.instance.onError` points to
  `WalletErrorHandler.handleError`, which captures and then exits.
- Flutter Sentry is enabled only when `SENTRY_DSN` is compiled in through Dart
  defines.
- `beforeSend` currently strips all breadcrumbs and removes exception values
  from normal Flutter events.

### Rust

- `wallet_core/flutter_api/src/api/full.rs` calls `init_logging()` and
  `init_sentry()` during `WalletCore.init()`.
- Rust Sentry initialization depends on compile-time `SENTRY_DSN` and
  `SENTRY_ENVIRONMENT`.
- Rust release naming currently comes from `release_name!()` rather than the app
  release identifier.
- Rust already emits categorized error events through `#[sentry_capture_error]`.
- Rust panic capture works through the Rust SDK and panic integration.
- Rust scrubbing removes breadcrumbs, transactions, and request data from
  outgoing events.

### Native iOS / Android

- There is no explicit native Sentry bootstrap in the app-owned startup path:
    - Android: `MainActivity` / `PlatformSupportInitializer`
    - iOS: `AppDelegate`
- Native crash and hang behavior therefore depends on the current Flutter plugin
  hybrid/native setup rather than an app-owned native configuration.

### Build, Release, and Symbols

- CI forwards `SENTRY_DSN`, `SENTRY_ENVIRONMENT`, and `SENTRY_RELEASE` into
  Flutter builds for Android and iOS.
- Fastlane already runs `sentry_dart_plugin` when `SENTRY_AUTH_TOKEN` is
  available, so app/native symbol upload is already part of the CI path.
- Rust does not currently use the same release identifier as Flutter/native.
- There is no explicit Android Rust debug-symbol upload path aligned with app
  releases.

## ONT Baseline Findings

These findings are from the ONT iOS and Android release builds exercised on
2026-06-10 using temporary validation hooks in this branch. Those hooks are only
validation tooling and are not part of the intended steady-state architecture.

| Scenario                   | Expected artifact              | Actual artifact                                                                                                                | Assessment                                                                                                                    |
| -------------------------- | ------------------------------ | ------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------- |
| Flutter uncaught exception | Uncaught Dart exception event  | No dedicated uncaught Dart exception event was found.                                                                          | Flutter uncaught capture is not producing a clean exception-quality artifact in the current setup.                            |
| Rust panic                 | Fatal Rust panic event         | Fatal panic issue [`NL-WALLET-ONT-98`](https://ictu-nl-wallet-qw.sentry.io/issues/7541575597/)                                 | Best current result. Rust panic capture works end-to-end.                                                                     |
| Rust categorized error     | Rust categorized error event   | Rust error issue [`NL-WALLET-ONT-9C`](https://ictu-nl-wallet-qw.sentry.io/issues/7541885148/)                                  | The `#[sentry_capture_error]` path works and can be used as a non-crashing validation case.                                   |
| Android native crash       | Native Android crash event     | Native crash issue [`NL-WALLET-ONT-9E`](https://ictu-nl-wallet-qw.sentry.io/issues/7542188871/)                                | Android native crash capture works through the current hybrid setup.                                                          |
| Android ANR                | Native Android ANR event       | Fatal ANR issue [`NL-WALLET-ONT-9G`](https://ictu-nl-wallet-qw.sentry.io/issues/7542278948/)                                   | Android ANR capture works when the app is left hung until Android terminates it and the app is relaunched.                    |
| iOS native crash           | Native iOS crash event         | Native iOS crash issue [`NL-WALLET-ONT-9H`](https://ictu-nl-wallet-qw.sentry.io/issues/7542535050/)                            | iOS native crash capture works after fixing the temporary validation hook.                                                    |
| iOS app hang               | Native app-hang/watchdog event | Native watchdog termination in shared issue bucket [`NL-WALLET-ONT-C`](https://ictu-nl-wallet-qw.sentry.io/issues/5558424923/) | iOS app-hang capture works, but validation depends on event context because watchdog terminations group into a shared bucket. |

Notes:

- The Rust panic landed with `environment=ont`, which is correct, but its
  release was `flutter_api@0.6.0-dev`, while the app release is
  `nl.ictu.edi.wallet.latest@0.6.0+0.6.0`. Release alignment is currently
  broken.
- On Android 11+ Sentry uses the OS `ApplicationExitInfo` ANR path. A short
  recoverable stall that resumes without process termination does not produce a
  dedicated ANR event; the process must be terminated for ANR and the next app
  launch flushes the artifact.

## Gap Analysis

| Area                        | Desired state                                                                    | Current state                                                                                                                                                           | Gap / impact                                                                                                              |
| --------------------------- | -------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| Startup coverage            | Telemetry active before `WalletCore` initialization                              | `WalletCore.init()` and `postInit()` happen before `SentryFlutter.init()`                                                                                               | Early failures can occur before Flutter telemetry exists.                                                                 |
| Failure-domain ownership    | One SDK owns each crash/exception domain                                         | Ownership is mixed and partly implicit                                                                                                                                  | Missing events and duplicate capture are hard to reason about.                                                            |
| Flutter uncaught exceptions | Uncaught Dart failures produce one clear exception event                         | Baseline did not produce a dedicated uncaught Dart exception event                                                                                                      | Current Flutter fatal-path capture is not reliable enough for validation.                                                 |
| Native crash artifacts      | iOS/Android native crashes, ANRs, and app hangs are captured as native artifacts | Android native crash, Android ANR, iOS native crash, and iOS app hang are captured through the current hybrid setup, but iOS app hangs land in a shared watchdog bucket | Native-layer coverage exists, but iOS app-hang attribution depends on event context rather than dedicated issue grouping. |
| Release and symbolication   | All relevant events use aligned releases and symbolication                       | App/native symbols upload exists; Rust release naming and Rust debug-symbol handling are separate                                                                       | Cross-layer correlation and Rust postmortem analysis are weakened.                                                        |
| Breadcrumbs                 | Safe, high-value breadcrumbs survive across Flutter, Rust, and native            | Flutter and Rust currently delete breadcrumbs; native has no app-owned policy                                                                                           | The app currently discards most pre-failure context.                                                                      |
| Operational context         | Only curated breadcrumbs and targeted low-volume events are sent                 | Local logs mostly stay local and there is no explicit cross-layer breadcrumb model                                                                                      | Sentry lacks structured runtime context without resorting to broad log shipping.                                          |
| Validation                  | Each trigger produces one identifiable artifact for its layer                    | Android 11+ ANR requires a process-terminating validation path, and iOS app hangs group into a shared watchdog bucket                                                   | Validation of per-run attribution is still incomplete.                                                                    |

## Dev Work

- Move Flutter telemetry bootstrap ahead of `WalletCore.init()` / `postInit()`.
- Keep one Flutter uncaught-error capture path and simplify the Dart fatal path
  to one owner.
- Initialize native telemetry explicitly on Android and iOS in app-owned startup
  code.
- Keep native Sentry ownership in the app shells and out of `platform_support`.
- Feed native init from the same build inputs as Flutter.
- Start sending native crash-file / minidump-style artifacts for native crash
  paths where the SDK/platform supports them.
- Update Rust `init_sentry()` to prefer compile-time `SENTRY_RELEASE` when
  present.
- Align config and release naming across Flutter, native, and Rust.
- Add a small curated breadcrumb convention and keep the initial emitter set
  intentionally small.
- Forward recent Flutter and Rust breadcrumbs into native scope.
- Replace blanket breadcrumb deletion with curated retention.
- Set the same `maxBreadcrumbs` value across Flutter, native, and Rust.
- Keep the current app/native symbol upload path in CI and make it an explicit
  release requirement.
- Treat the app dSYM path as the iOS Rust symbolication path.
- Add explicit Android Rust debug-symbol upload for `libwallet_core.so` in the
  existing release jobs.
- Stop sending user geo context by disabling automatic geo enrichment and
  stripping geo fields from outgoing events where needed.
- Keep temporary validation hooks only in non-production validation builds until
  rollout verification is complete.

## Manual Tests

- Validate on a non-production DSN/environment:
    - Flutter uncaught exception captured once with Dart stacktrace
    - Rust panic captured once with Rust stacktrace and aligned release
    - Rust categorized error captured once without crashing the app
    - Android native crash captured once with native symbolication
    - Android ANR captured once after system termination and relaunch on Android
      11+
    - iOS native crash captured once with native symbolication
    - iOS app hang captured once
    - explicit breadcrumb trail visible on relevant events with the expected
      `wallet.<category>` shape
    - no duplicate captures for the same fatal path
