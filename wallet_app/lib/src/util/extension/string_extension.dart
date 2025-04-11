import 'package:flutter/cupertino.dart';
import 'package:flutter/rendering.dart';

import '../../domain/model/localized_text.dart';
import 'build_context_extension.dart';

extension StringExtension on String {
  /// Capitalizes first letter in string (if present)
  String get capitalize => isNotEmpty ? '${this[0].toUpperCase()}${substring(1).toLowerCase()}' : '';

  /// Removes last character from string (if present)
  String get removeLastChar => length <= 1 ? '' : substring(0, length - 1);

  /// Adds space to end of string (if not empty)
  String get addSpaceSuffix => isNotEmpty ? '$this ' : '';

  LocalizedText get untranslated => {Locale('en'): this};

  AttributedString toAttributedString(BuildContext context) => AttributedString(
        this,
        attributes: [
          LocaleStringAttribute(
            range: fullRange,
            locale: context.activeLocale,
          ),
        ],
      );

  TextSpan toTextSpan(BuildContext context) => TextSpan(text: this, locale: context.activeLocale);

  TextRange get fullRange => TextRange(start: 0, end: length);
}
