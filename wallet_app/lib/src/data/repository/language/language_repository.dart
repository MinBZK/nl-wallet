import 'dart:ui';

abstract class LanguageRepository {
  Future<List<Locale>> getAvailableLocales();

  Future<void> setPreferredLocale(Locale locale);

  Stream<Locale?> get preferredLocale;
}
