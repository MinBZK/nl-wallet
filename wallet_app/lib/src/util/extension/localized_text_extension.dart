import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';

import '../../../environment.dart';
import '../../domain/model/localized_text.dart';
import 'build_context_extension.dart';
import 'locale_extension.dart';

extension LocalizedLabelsExtension on LocalizedText {
  /// Retrieve the most relevant translation based on the active locale
  String l10nValue(BuildContext context) => l10nValueForLanguageCode(context.localeName);

  TextSpan l10nSpan(BuildContext context) =>
      TextSpan(text: l10nValueForLanguageCode(context.localeName), locale: localeForLanguageCode(context.localeName));

  /// Retrieve the entry for the provided languageCode, falling back to a sane default if none is found.
  /// Fallback logic:
  /// 1. Select the english translation
  /// 2. Provide any (the first) available translation
  /// 3. Return null
  MapEntry<String, String>? resolveEntryForLanguageCode(String languageCode) {
    // Resolve the correct locale
    for (final entry in entries) {
      final availableLocale = LocaleExtension.tryParseLocale(entry.key);
      if (availableLocale?.languageCode == languageCode) return entry;
    }

    // Fallback to english locale
    for (final entry in entries) {
      final availableLocale = LocaleExtension.tryParseLocale(entry.key);
      if (availableLocale?.languageCode == 'en') return entry;
    }

    // Fallback to any available locale, or empty entry if LocalizedText is empty.
    Fimber.d('Could not resolve localized value for: $this');
    return entries.firstOrNull;
  }

  // Resolve the most appropriate localization for the provided language code, falling back to an empty string if none is found.
  String l10nValueForLanguageCode(String languageCode) => resolveEntryForLanguageCode(languageCode)?.value ?? '';

  // Resolve the most relevant locale based on the provided language code, matching the logic used by [l10nValueForLanguageCode].
  Locale? localeForLanguageCode(String languageCode) {
    final locale = resolveEntryForLanguageCode(languageCode)?.key;
    if (locale == null || locale.isEmpty) return null;
    return LocaleExtension.tryParseLocale(locale);
  }

  String get testValue {
    assert(
      Environment.isTest,
      'This should never be used in real builds, as a BuildContext should readily be available in that case',
    );
    return this['en'] ?? values.firstOrNull ?? '';
  }
}
