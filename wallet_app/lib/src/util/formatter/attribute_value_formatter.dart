import 'package:fimber/fimber.dart';
import 'package:flutter/cupertino.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:intl/intl.dart';

import '../../domain/model/attribute/attribute.dart';
import '../../domain/model/attribute/value/gender.dart';
import '../extension/build_context_extension.dart';

class AttributeValueFormatter {
  static String format(BuildContext context, AttributeValue attributeValue) {
    return switch (attributeValue) {
      StringValue() => attributeValue.value,
      BooleanValue() => attributeValue.value ? context.l10n.cardValueTrue : context.l10n.cardValueFalse,
      DateValue() => _prettyPrintDateTime(context.localeName, attributeValue.value),
      GenderValue() => _prettyPrintGender(context.l10n, attributeValue.value),
    };
  }

  static String formatWithLocale(Locale locale, AttributeValue attributeValue) {
    final l10n = lookupAppLocalizations(locale);
    return switch (attributeValue) {
      StringValue() => attributeValue.value,
      BooleanValue() => attributeValue.value ? l10n.cardValueTrue : l10n.cardValueFalse,
      DateValue() => _prettyPrintDateTime(locale.languageCode, attributeValue.value),
      GenderValue() => _prettyPrintGender(l10n, attributeValue.value),
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

  static String _prettyPrintGender(AppLocalizations l10n, Gender gender) {
    switch (gender) {
      case Gender.unknown:
        return l10n.cardValueGenderUnknown;
      case Gender.male:
        return l10n.cardValueGenderMale;
      case Gender.female:
        return l10n.cardValueGenderFemale;
      case Gender.notApplicable:
        return l10n.cardValueGenderNotApplicable;
    }
  }
}

extension AttributeValueExtension on AttributeValue {
  String prettyPrint(BuildContext context) => AttributeValueFormatter.format(context, this);
}
