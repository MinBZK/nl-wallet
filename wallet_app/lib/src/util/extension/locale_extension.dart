import 'package:fimber/fimber.dart';
import 'package:flutter/widgets.dart';
import 'package:intl/locale.dart' as intl;

import '../../../l10n/generated/app_localizations.dart';
import '../../../l10n/generated/app_localizations_en.dart';
import 'build_context_extension.dart';

extension LocaleExtension on Locale {
  static Locale parseLocale(String rawLocale) {
    final intlLocale = intl.Locale.tryParse(rawLocale);
    if (intlLocale != null) {
      return Locale.fromSubtags(
        languageCode: intlLocale.languageCode,
        countryCode: intlLocale.countryCode,
        scriptCode: intlLocale.scriptCode,
      );
    }
    assert(rawLocale.isNotEmpty, 'Empty locales are not supported');
    Fimber.w('Failed to properly parse locale: $rawLocale');
    // Fallback to unparsed locale, this could result into unexpected behaviour.
    return Locale(rawLocale);
  }

  bool matchesCurrentLocale(BuildContext context) => this == context.activeLocale;

  bool matchesCurrentLanguage(BuildContext context) => languageCode == context.activeLocale.languageCode;

  AppLocalizations get l10n {
    try {
      return lookupAppLocalizations(this);
    } catch (ex) {
      Fimber.e('Failed to resolve l10n for locale: $this. Falling back to english.', ex: ex);
      return AppLocalizationsEn();
    }
  }
}
