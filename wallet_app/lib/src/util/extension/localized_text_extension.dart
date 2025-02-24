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
      TextSpan(text: l10nValueForLanguageCode(context.localeName), locale: _resolveSelectedLocale(context));

  /// Match the fallback logic of [l10nValueForLanguageCode]
  Locale _resolveSelectedLocale(BuildContext context) {
    try {
      if (this[context.localeName] != null) return Locale(context.localeName);
      if (this['en'] != null) return const Locale('en');
      return Locale(keys.firstOrNull ?? context.activeLocale.languageCode);
    } catch (ex) {
      Fimber.e('Failed to resolve locale for $this', ex: ex);
      return context.activeLocale;
    }
  }

  /// Retrieve the translation for the provided languageCode, falling back to a sane default if none it found.
  /// Fallback logic:
  /// 1. Select the english translation
  /// 2. Provide any (the first) available translation
  /// 3. Return an empty string
  String l10nValueForLanguageCode(String languageCode) {
    // Resolve the correct locale
    for (final entry in entries) {
      final availableLocale = LocaleExtension.tryParseLocale(entry.key);
      if (availableLocale?.languageCode == languageCode) return entry.value;
    }

    // Fallback to english locale
    for (final entry in entries) {
      final availableLocale = LocaleExtension.tryParseLocale(entry.key);
      if (availableLocale?.languageCode == 'en') return entry.value;
    }

    // Fallback to any available locale, or empty string if LocalizedText is empty.
    Fimber.d('Could not resolve localized value for: $this');
    return values.firstOrNull ?? '';
  }

  String get testValue {
    assert(
      Environment.isTest,
      'This should never be used in real builds, as a BuildContext should readily be available in that case',
    );
    return this['en'] ?? values.firstOrNull ?? '';
  }
}
