import 'dart:io';

class Environment {
  Environment._();

  static bool get mockRepositories => const bool.fromEnvironment('MOCK_REPOSITORIES', defaultValue: false);

  static bool get isTest => Platform.environment.containsKey('FLUTTER_TEST');

  static bool get isMockOrTest => mockRepositories || isTest;

  static String get demoRelyingPartyUrl => const String.fromEnvironment('DEMO_INDEX_URL');

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
