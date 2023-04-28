import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:intl/intl.dart';

class OperationIssuedTimeFormatter {
  static String format(AppLocalizations locale, DateTime dateTime) {
    DateFormat dateTimeFormat = DateFormat(DateFormat.MONTH_DAY, locale.localeName).add_Hm();
    return dateTimeFormat.format(dateTime);
  }
}
