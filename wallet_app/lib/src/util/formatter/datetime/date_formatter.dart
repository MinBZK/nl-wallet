import 'package:flutter/cupertino.dart';
import 'package:intl/intl.dart';

import '../../extension/build_context_extension.dart';

class DateFormatter {
  static String formatDate(BuildContext context, DateTime datetime) {
    final locale = context.activeLocale;
    return DateFormat.yMMMMd(locale.toLanguageTag()).format(datetime);
  }

  static String formatDateTime(BuildContext context, DateTime datetime) {
    final locale = context.activeLocale;
    final dateFormatted = formatDate(context, datetime);
    final timeFormatted = DateFormat.Hm(locale.toLanguageTag()).format(datetime);
    return '$dateFormatted, $timeFormatted';
  }
}
