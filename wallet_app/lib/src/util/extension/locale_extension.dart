import 'package:flutter/cupertino.dart';
import 'package:intl/locale.dart' as intl;

import 'build_context_extension.dart';

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

  bool matchesCurrentLanguage(BuildContext context) => languageCode == context.localeName;
}
