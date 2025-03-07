import 'package:fimber/fimber.dart';
import 'package:flutter/cupertino.dart';
import 'package:intl/intl.dart';

import '../../../l10n/generated/app_localizations.dart';
import '../../domain/model/attribute/attribute.dart';
import '../extension/build_context_extension.dart';

class AttributeValueFormatter {
  static String format(BuildContext context, AttributeValue attributeValue) {
    return switch (attributeValue) {
      StringValue() => attributeValue.value,
      BooleanValue() => attributeValue.value ? context.l10n.cardValueTrue : context.l10n.cardValueFalse,
      NumberValue() => attributeValue.value.toString(),
      DateValue() => _prettyPrintDateTime(context.localeName, attributeValue.value),
    };
  }

  static String formatWithLocale(Locale locale, AttributeValue attributeValue) {
    final l10n = lookupAppLocalizations(locale);
    return switch (attributeValue) {
      StringValue() => attributeValue.value,
      BooleanValue() => attributeValue.value ? l10n.cardValueTrue : l10n.cardValueFalse,
      NumberValue() => attributeValue.value.toString(),
      DateValue() => _prettyPrintDateTime(locale.languageCode, attributeValue.value),
    };
  }

  static String _prettyPrintDateTime(String locale, DateTime dateTime) {
    if (DateFormat.localeExists(locale)) {
      return DateFormat(DateFormat.YEAR_MONTH_DAY, locale).format(dateTime);
    } else {
      Fimber.i('DateFormat does not support locale: $locale, formatting without locale.');
      return DateFormat(DateFormat.YEAR_MONTH_DAY).format(dateTime);
    }
  }
}

extension AttributeValueExtension on AttributeValue {
  String prettyPrint(BuildContext context) => AttributeValueFormatter.format(context, this);
}
