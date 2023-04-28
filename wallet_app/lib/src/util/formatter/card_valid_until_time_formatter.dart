import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:intl/intl.dart';

class CardValidUntilTimeFormatter {
  static String format(AppLocalizations locale, DateTime dateTime) {
    DateFormat dateTimeFormat = DateFormat(DateFormat.YEAR_MONTH_DAY, locale.localeName);
    return dateTimeFormat.format(dateTime);
  }
}
