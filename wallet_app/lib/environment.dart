import 'dart:io';

class Environment {
  Environment._();

  static bool get mockRepositories => const bool.fromEnvironment('MOCK_REPOSITORIES', defaultValue: true);

  static bool get isTest => Platform.environment.containsKey('FLUTTER_TEST');

  static String get sentryDsn => const String.fromEnvironment('SENTRY_DSN');

  static bool get hasSentryDsn => sentryDsn.isNotEmpty;
}
