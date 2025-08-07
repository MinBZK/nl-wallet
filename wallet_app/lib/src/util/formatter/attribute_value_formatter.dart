import 'package:flutter/cupertino.dart';
import 'package:intl/intl.dart';

import '../../domain/model/attribute/attribute.dart';
import '../extension/build_context_extension.dart';
import '../extension/locale_extension.dart';

class AttributeValueFormatter {
  static String format(BuildContext context, AttributeValue attributeValue) =>
      formatWithLocale(context.activeLocale, attributeValue);

  static String formatWithLocale(Locale locale, AttributeValue attribute) {
    final l10n = locale.l10n;
    return switch (attribute) {
      StringValue() => attribute.value.isEmpty ? l10n.cardValueEmpty : attribute.value,
      BooleanValue() => attribute.value ? l10n.cardValueTrue : l10n.cardValueFalse,
      NumberValue() => '${attribute.value}',
      DateValue() => _prettyPrintDateTime(locale, attribute.value),
      ArrayValue() => _formatArrayValue(locale, attribute),
      NullValue() => l10n.cardValueNull,
    };
  }

  static String _formatArrayValue(Locale locale, ArrayValue attribute) {
    if (attribute.value.isEmpty) return locale.l10n.cardValueEmptyList;
    return attribute.value.map((it) => '  • ${formatWithLocale(locale, it)}').join('\n');
  }

  static String _prettyPrintDateTime(Locale locale, DateTime dateTime) {
    return DateFormat.yMd(locale.toLanguageTag()).format(dateTime);
  }
}

extension AttributeValueExtension on AttributeValue {
  String prettyPrint(BuildContext context) => AttributeValueFormatter.format(context, this);
}
