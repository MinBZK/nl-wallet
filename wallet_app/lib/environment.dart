import 'dart:io';

class Environment {
  Environment._();

  static bool get mockRepositories => const bool.fromEnvironment('MOCK_REPOSITORIES', defaultValue: false);

  static bool get isTest => Platform.environment.containsKey('FLUTTER_TEST');

  static String get mockRelyingPartyUrl => const String.fromEnvironment('MOCK_RELYING_PARTY_URL');

  static String get sentryDsn => const String.fromEnvironment('SENTRY_DSN');

  static bool get hasSentryDsn => sentryDsn.isNotEmpty;

  static String get sentryEnvironment =>
      const String.fromEnvironment('SENTRY_ENVIRONMENT', defaultValue: 'unspecified');

  static String? sentryRelease() {
    const release = String.fromEnvironment('SENTRY_RELEASE', defaultValue: 'unspecified');
    if (release == 'unspecified') {
      return null;
    } else {
      return release;
    }
  }
}
