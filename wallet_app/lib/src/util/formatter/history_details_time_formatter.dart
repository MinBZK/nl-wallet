import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:intl/intl.dart';

class HistoryDetailsTimeFormatter {
  static String format(AppLocalizations locale, DateTime dateTime) {
    DateFormat dateTimeFormat = DateFormat('d MMMM y, HH:mm', locale.localeName);
    return dateTimeFormat.format(dateTime);
  }
}
