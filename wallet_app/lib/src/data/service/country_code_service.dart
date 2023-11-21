import 'package:country_codes/country_codes.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';

import '../store/active_locale_provider.dart';

/// Service that makes sure the [CountryCodes] package is initialized for the selected app language.
class CountryCodeService {
  CountryCodeService(ActiveLocaleProvider localeProvider) {
    localeProvider.observe().listen(_onLocaleChanged);
  }

  void _onLocaleChanged(Locale locale) async {
    final result = await CountryCodes.init(locale);
    if (result) {
      Fimber.d('CountryCodes successfully updated to locale: $locale');
    } else {
      Fimber.e('CountryCodes failed to update to locale: $locale');
    }
  }
}
