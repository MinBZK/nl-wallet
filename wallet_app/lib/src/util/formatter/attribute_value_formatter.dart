import 'package:fimber/fimber.dart';
import 'package:flutter/cupertino.dart';
import 'package:intl/intl.dart';

import '../../../l10n/generated/app_localizations.dart';
import '../../../l10n/generated/app_localizations_en.dart';
import '../../domain/model/attribute/attribute.dart';
import '../extension/build_context_extension.dart';

class AttributeValueFormatter {
  static String format(BuildContext context, AttributeValue attributeValue) {
    return switch (attributeValue) {
      StringValue() => attributeValue.value,
      BooleanValue() => attributeValue.value ? context.l10n.cardValueTrue : context.l10n.cardValueFalse,
      NumberValue() => attributeValue.value.toString(),
      DateValue() => _prettyPrintDateTime(context.activeLocale, attributeValue.value),
    };
  }

  static String formatWithLocale(Locale locale, AttributeValue attributeValue) {
    late AppLocalizations l10n;
    try {
      l10n = lookupAppLocalizations(locale);
    } catch (ex) {
      Fimber.e('Failed to resolve l10n for locale: $locale. Falling back to english.', ex: ex);
      l10n = AppLocalizationsEn();
    }
    return switch (attributeValue) {
      StringValue() => attributeValue.value,
      BooleanValue() => attributeValue.value ? l10n.cardValueTrue : l10n.cardValueFalse,
      NumberValue() => attributeValue.value.toString(),
      DateValue() => _prettyPrintDateTime(locale, attributeValue.value),
    };
  }

  static String _prettyPrintDateTime(Locale locale, DateTime dateTime) {
    return DateFormat.yMd(locale.toLanguageTag()).format(dateTime);
  }
}

extension AttributeValueExtension on AttributeValue {
  String prettyPrint(BuildContext context) => AttributeValueFormatter.format(context, this);
}
