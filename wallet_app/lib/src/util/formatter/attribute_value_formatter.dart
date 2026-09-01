import 'package:flutter/cupertino.dart';
import 'package:intl/intl.dart';

import '../../domain/model/attribute/attribute.dart';
import '../extension/build_context_extension.dart';
import '../extension/locale_extension.dart';

class AttributeValueFormatter {
  static String format(BuildContext context, AttributeValue attributeValue, {bool inline = false}) =>
      formatWithLocale(context.activeLocale, attributeValue, inline: inline);

  /// Set [inline] for single line contexts, such as the summary on a card front. Composite values are
  /// then joined with a comma instead of being spread over multiple lines.
  static String formatWithLocale(Locale locale, AttributeValue attribute, {bool inline = false}) {
    final l10n = locale.l10n;
    return switch (attribute) {
      StringValue() => attribute.value.isEmpty ? l10n.cardValueEmpty : attribute.value,
      BooleanValue() => attribute.value ? l10n.cardValueTrue : l10n.cardValueFalse,
      NumberValue() => '${attribute.value}',
      DateValue() => _prettyPrintDateTime(locale, attribute.value),
      ArrayValue() => _formatArrayValue(locale, attribute, inline),
      BytesValue() => l10n.cardValueUnsupported,
      ImageValue() => l10n.cardValueImage,
      MapValue() => _formatMapValue(locale, attribute, inline),
      NullValue() => l10n.cardValueNull,
    };
  }

  static String _formatArrayValue(Locale locale, ArrayValue attribute, bool inline) {
    if (attribute.value.isEmpty) return locale.l10n.cardValueEmptyList;
    final entries = attribute.value.map((it) => formatWithLocale(locale, it, inline: inline));
    return inline ? entries.join(', ') : entries.map((it) => '  • $it').join('\n');
  }

  static String _formatMapValue(Locale locale, MapValue attribute, bool inline) {
    if (attribute.value.isEmpty) return locale.l10n.cardValueEmptyList;
    final entries = attribute.value.entries.map(
      (it) => '${it.key}: ${formatWithLocale(locale, it.value, inline: inline)}',
    );
    return entries.join(inline ? ', ' : '\n');
  }

  static String _prettyPrintDateTime(Locale locale, DateTime dateTime) {
    return DateFormat.yMd(locale.toLanguageTag()).format(dateTime);
  }
}

extension AttributeValueExtension on AttributeValue {
  String prettyPrint(BuildContext context, {bool inline = false}) =>
      AttributeValueFormatter.format(context, this, inline: inline);
}
