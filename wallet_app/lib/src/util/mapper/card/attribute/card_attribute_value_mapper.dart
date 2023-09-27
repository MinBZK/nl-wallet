import 'dart:ui';

import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:intl/intl.dart';

import '../../../../wallet_core/wallet_core.dart';

class CardAttributeValueMapper {
  CardAttributeValueMapper();

  String map(CardValue input, Locale locale) {
    return input.map(
      string: (input) => input.value,
      boolean: (input) {
        final l10n = lookupAppLocalizations(locale);
        return input.value ? l10n.cardValueTrue : l10n.cardValueFalse;
      },
      date: (input) {
        final date = DateTime.parse(input.value);
        return DateFormat(DateFormat.YEAR_MONTH_DAY, locale.toString()).format(date);
      },
      gender: (input) {
        final l10n = lookupAppLocalizations(locale);
        switch (input.value) {
          case GenderCardValue.Unknown:
            return l10n.cardValueGenderUnknown;
          case GenderCardValue.Male:
            return l10n.cardValueGenderMale;
          case GenderCardValue.Female:
            return l10n.cardValueGenderFemale;
          case GenderCardValue.NotApplicable:
            return l10n.cardValueGenderNotApplicable;
        }
      },
    );
  }
}
