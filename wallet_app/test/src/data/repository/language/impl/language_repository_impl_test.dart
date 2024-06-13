import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/data/repository/language/impl/language_repository_impl.dart';
import 'package:wallet/src/data/repository/language/language_repository.dart';
import 'package:wallet/src/data/store/language_store.dart';

void main() {
  final _MockLanguageStore languageStore = _MockLanguageStore();
  const defaultLocales = [Locale('nl'), Locale('en')];
  late LanguageRepository languageRepository;

  setUp(() {
    languageRepository = LanguageRepositoryImpl(languageStore, defaultLocales);
  });

  group('preferredLocale & setPreferredLocale', () {
    test('should default to null when instantiated', () async {
      expect(await languageRepository.preferredLocale.first, isNull);
    });

    test('should emit updated locale when it is set', () async {
      const nlLocale = Locale('nl');
      const enLocale = Locale('en');
      expect(languageRepository.preferredLocale, emitsInOrder([isNull, nlLocale, enLocale]));
      await languageRepository.setPreferredLocale(nlLocale);
      await languageRepository.setPreferredLocale(enLocale);
    });

    test('should not emit null locale if locale was previously set', () async {
      const nlLocale = Locale('nl');
      await languageRepository.setPreferredLocale(nlLocale);
      expect(languageRepository.preferredLocale, emitsInOrder([nlLocale]));
    });
  });

  group('getAvailableLocales', () {
    test('should return the provided defaultLocales when requested', () async {
      expect(await languageRepository.getAvailableLocales(), defaultLocales);
    });
  });
}

class _MockLanguageStore implements LanguageStore {
  String? preferredLanguageCode;

  @override
  Future<String?> getPreferredLanguageCode() async => preferredLanguageCode;

  @override
  Future<void> setPreferredLanguageCode(String? languageCode) async => preferredLanguageCode = languageCode;
}
