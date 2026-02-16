import 'package:flutter/cupertino.dart';
import 'package:intl/intl.dart';

import '../../extension/build_context_extension.dart';

class DateFormatter {
  /// Formats a [DateTime] to a localized date string.
  ///
  /// Example: "January 15, 2024" or "15 januari 2024" (format depends on locale).
  static String formatDate(BuildContext context, DateTime datetime) {
    return DateFormat.yMMMMd(context.l10n.localeName).format(datetime);
  }

  /// Formats a [DateTime] to a localized date and time string.
  ///
  /// Example: "January 15, 2024, 10:30" or "15 januari 2024, 22:30" (format depends on locale).
  static String formatDateTime(BuildContext context, DateTime datetime) {
    final dateFormatted = formatDate(context, datetime);
    final timeFormatted = DateFormat.Hm(context.l10n.localeName).format(datetime);
    return '$dateFormatted, $timeFormatted';
  }

  /// Formats a [DateTime] to a localized month and day string.
  ///
  /// Example: "January 15" (format depends on locale).
  static String formatMonthDay(BuildContext context, DateTime dateTime) {
    return DateFormat(DateFormat.MONTH_DAY, context.l10n.localeName).format(dateTime);
  }

  /// Formats a [DateTime] to a localized hour and minute string.
  ///
  /// Example: "10:30 AM" or "22:30" (format depends on locale).
  static String formatTime(BuildContext context, DateTime dateTime) {
    return DateFormat(DateFormat.HOUR_MINUTE, context.l10n.localeName).format(dateTime);
  }
}
