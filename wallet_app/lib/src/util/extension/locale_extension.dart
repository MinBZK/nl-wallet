import 'dart:ui';

import 'package:intl/locale.dart' as intl;

extension LocaleExtension on Locale {
  static Locale? tryParseLocale(String rawLocale) {
    final intlLocale = intl.Locale.tryParse(rawLocale);
    if (intlLocale != null) {
      return Locale.fromSubtags(
        languageCode: intlLocale.languageCode,
        countryCode: intlLocale.countryCode,
        scriptCode: intlLocale.scriptCode,
      );
    }
    return null;
  }
}
