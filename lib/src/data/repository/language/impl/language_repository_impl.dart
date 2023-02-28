import 'dart:ui';

import 'package:rxdart/rxdart.dart';

import '../../../store/language_store.dart';
import '../language_repository.dart';

class LanguageRepositoryImpl extends LanguageRepository {
  final List<Locale> _supportedLocales;
  final LanguageStore _languageStore;

  final BehaviorSubject<Locale?> _localeStream = BehaviorSubject.seeded(null);

  LanguageRepositoryImpl(this._languageStore, this._supportedLocales) {
    _languageStore.getPreferredLanguageCode().then((languageCode) {
      _localeStream.add(languageCode == null ? null : Locale(languageCode));
    });
  }

  @override
  Future<List<Locale>> getAvailableLocales() async => _supportedLocales;

  @override
  Future<void> setPreferredLocale(Locale? locale) async {
    _languageStore.setPreferredLanguageCode(locale?.languageCode);
    _localeStream.add(locale);
  }

  @override
  Stream<Locale?> get preferredLocale => _localeStream.distinct();
}
