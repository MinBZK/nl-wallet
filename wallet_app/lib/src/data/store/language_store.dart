abstract class LanguageStore {
  Future<String?> getPreferredLanguageCode();

  Future<void> setPreferredLanguageCode(String? languageCode);
}
