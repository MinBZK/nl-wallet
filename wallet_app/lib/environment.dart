import 'dart:io';

import 'package:flutter/foundation.dart';

/// A utility class for accessing environment-specific configurations.
///
/// This class provides static getters to retrieve values defined at compile-time.
class Environment {
  Environment._();

  /// Indicates whether mock repositories should be used. Defaults to false.
  ///
  /// Set using: `flutter build --dart-define=MOCK_REPOSITORIES=true`
  static bool get mockRepositories => const bool.fromEnvironment('MOCK_REPOSITORIES', defaultValue: false);

  /// Indicates whether the application is currently running in a test environment.
  static bool get isTest => Platform.environment.containsKey('FLUTTER_TEST');

  /// Indicates whether the auto-lock feature should be disabled. Defaults to false.
  ///
  /// Auto-lock can only be disabled when the app is not running in release mode.
  /// Set using: `flutter build --dart-define=DISABLE_AUTO_LOCK=true`
  static bool get disableAutoLock =>
      const bool.fromEnvironment('DISABLE_AUTO_LOCK', defaultValue: false) && !kReleaseMode;

  /// A convenience getter that returns `true` if either `mockRepositories` or `isTest` is `true`.
  static bool get isMockOrTest => mockRepositories || isTest;

  /// The URL for the demo relying party.
  ///
  /// Set using: `flutter build --dart-define=DEMO_INDEX_URL=https://example.org/demo`
  static String get demoRelyingPartyUrl => const String.fromEnvironment('DEMO_INDEX_URL');

  /// The DSN for Sentry error reporting.
  ///
  /// Set using: `flutter build --dart-define=SENTRY_DSN=your_sentry_dsn`
  static String get sentryDsn => const String.fromEnvironment('SENTRY_DSN');

  /// Indicates whether a Sentry DSN has been provided.
  static bool get hasSentryDsn => sentryDsn.isNotEmpty;

  /// The environment name for Sentry (e.g., "development", "production").
  ///
  /// Set using: `flutter build --dart-define=SENTRY_ENVIRONMENT=production`
  static String get sentryEnvironment =>
      const String.fromEnvironment('SENTRY_ENVIRONMENT', defaultValue: 'unspecified');

  /// The release version for Sentry.
  ///
  /// Returns null when unset.
  /// Set using: `flutter build --dart-define=SENTRY_RELEASE=1.0.0`
  static String? sentryRelease() {
    const release = String.fromEnvironment('SENTRY_RELEASE', defaultValue: 'unspecified');
    if (release == 'unspecified') {
      return null;
    } else {
      return release;
    }
  }
}
