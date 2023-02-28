class Environment {
  Environment._();

  static bool get mockRepositories => const bool.fromEnvironment('MOCK_REPOSITORIES', defaultValue: true);
}
