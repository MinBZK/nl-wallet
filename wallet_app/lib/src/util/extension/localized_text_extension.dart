import 'package:flutter/material.dart';

import '../../../environment.dart';
import '../../domain/model/localized_text.dart';
import 'build_context_extension.dart';

extension LocalizedLabelsExtension on LocalizedText {
  /// Retrieve the most relevant translation based on the active locale
  String l10nValue(BuildContext context) => l10nValueFromLocale(context.locale);

  /// Retrieve the translation for the provided languageCode, falling back to a sane default if none it found.
  /// Fallback logic:
  /// 1. Select the english translation
  /// 2. Provide any (the first) available translation
  /// 3. Return an empty string
  String l10nValueFromLocale(String languageCode) => this[languageCode] ?? this['en'] ?? values.firstOrNull ?? '';

  String get testValue {
    assert(Environment.isTest,
        'This should never be used in real builds, as a BuildContext should readily be available in that case');
    return this['en'] ?? values.firstOrNull ?? '';
  }
}
