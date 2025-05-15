import 'package:fimber/fimber.dart';
import 'package:flutter/widgets.dart';

import '../../../environment.dart';
import '../../domain/model/localized_text.dart';
import 'build_context_extension.dart';

extension LocalizedTextExtension on LocalizedText {
  /// Retrieve the most relevant translation based on the active locale
  String l10nValue(BuildContext context) => l10nValueForLocale(context.activeLocale);

  TextSpan l10nSpan(BuildContext context) =>
      TextSpan(text: l10nValueForLocale(context.activeLocale), locale: context.activeLocale);

  /// Retrieve the entry for the provided languageCode, falling back to a sane default if none is found.
  /// Fallback logic:
  /// 1. Select the english translation
  /// 2. Provide any (the first) available translation
  /// 3. Return null
  MapEntry<Locale, String>? resolveEntryForLocale(Locale locale) {
    // Resolve the correct locale
    for (final entry in entries) {
      if (entry.key == locale) return entry;
    }
    // Resolve locale solely by languageTag
    for (final entry in entries) {
      if (entry.key.languageCode == locale.languageCode) return entry;
    }

    // Fallback to english locale
    for (final entry in entries) {
      if (entry.key.languageCode == 'en') return entry;
    }

    // Fallback to any available locale, or empty entry if LocalizedText is empty.
    Fimber.d('Could not resolve localized value for: $this');
    return entries.firstOrNull;
  }

  // Resolve the most appropriate localization for the provided language code, falling back to an empty string if none is found.
  String l10nValueForLocale(Locale locale) => resolveEntryForLocale(locale)?.value ?? '';

  // Resolve the most relevant locale based on the provided language code, matching the logic used by [resolveEntryForLocale].
  Locale? localeForLanguageCode(String languageCode) {
    return resolveEntryForLocale(Locale(languageCode))?.key;
  }

  @visibleForTesting
  String get testValue {
    assert(
      Environment.isTest,
      'This should never be used in real builds, as a BuildContext should readily be available in that case',
    );
    return resolveEntryForLocale(Locale('en'))?.value ?? '';
  }
}
