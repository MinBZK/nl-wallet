import 'package:flutter/cupertino.dart';
import 'package:intl/intl.dart';

import '../extension/build_context_extension.dart';

class TimeAgoFormatter {
  static String format(BuildContext context, DateTime dateTime) {
    final difference = DateTime.now().difference(dateTime);

    if (difference.inDays >= DateTime.daysPerWeek) {
      DateFormat dateFormat = DateFormat(DateFormat.MONTH_DAY, context.l10n.localeName);
      return dateFormat.format(dateTime);
    } else {
      String time = '';
      if (difference.inDays >= 1) {
        time = context.l10n.generalDays(difference.inDays);
      } else if (difference.inHours >= 1) {
        time = context.l10n.generalHours(difference.inHours);
      } else if (difference.inMinutes >= 1) {
        time = context.l10n.generalMinutes(difference.inMinutes);
      } else {
        time = context.l10n.generalTimeAgoLessThenOneMinute;
      }
      return '$time ${context.l10n.generalTimeAgo}';
    }
  }
}
