import 'package:flutter/material.dart';
import 'package:wallet/l10n/generated/app_localizations.dart';
import 'package:rxdart/rxdart.dart';

import '../active_locale_provider.dart';

/// A [LocalizationsDelegate] who's sole purpose it is to expose the resolved localization, which it does by implementing
/// the [ActiveLocaleProvider]. Note that this [ActiveLocalizationDelegate] has to be added to the list of
/// [MaterialApp.localizationsDelegates] for it to work.
class ActiveLocalizationDelegate extends LocalizationsDelegate<void> implements ActiveLocaleProvider {
  ActiveLocalizationDelegate();

  /// Stream containing the active locale, seeded with a sane default to avoid nullability
  final BehaviorSubject<Locale> _activeLocaleStream = BehaviorSubject.seeded(AppLocalizations.supportedLocales.first);

  @override
  Stream<Locale> observe() => _activeLocaleStream.stream;

  @override
  Locale get activeLocale => _activeLocaleStream.value;

  @override
  bool isSupported(Locale locale) => true;

  @override
  Future<void> load(Locale locale) async => _activeLocaleStream.add(locale);

  @override
  bool shouldReload(ActiveLocalizationDelegate old) => false;
}
