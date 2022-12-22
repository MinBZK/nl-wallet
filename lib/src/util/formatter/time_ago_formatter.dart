import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:intl/intl.dart';

class TimeAgoFormatter {
  static String format(AppLocalizations locale, DateTime dateTime) {
    final difference = DateTime.now().difference(dateTime);

    if (difference.inDays >= DateTime.daysPerWeek) {
      DateFormat dateFormat = DateFormat(DateFormat.MONTH_DAY, locale.localeName);
      return dateFormat.format(dateTime);
    } else {
      String time = '';
      if (difference.inDays >= 1) {
        time = locale.generalDays(difference.inDays);
      } else if (difference.inHours >= 1) {
        time = locale.generalHours(difference.inHours);
      } else if (difference.inMinutes >= 1) {
        time = locale.generalMinutes(difference.inMinutes);
      } else {
        time = locale.generalTimeAgoLessThenOneMinute;
      }
      return '$time ${locale.generalTimeAgo}';
    }
  }
}
