import 'package:clock/clock.dart';

import '../../../../l10n/generated/app_localizations.dart';

class DurationFormatter {
  static String prettyPrintTimeAgo(AppLocalizations l10n, DateTime dateTime) {
    final difference = dateTime.difference(clock.now());
    if (difference.isNegative) {
      /// [DateTime] is in the past, format time ago
      if (difference.inDays.abs() >= DateTime.daysPerWeek) {
        return l10n.generalTimeAgoMoreThanOneWeek;
      } else {
        final time = DurationFormatter.prettyPrintTimeDifference(l10n, dateTime);
        return '$time ${l10n.generalTimeAgo}';
      }
    } else {
      /// [DateTime] is in the future, fallback to "less than a minute ago"
      return '${l10n.generalTimeAgoLessThenOneMinute} ${l10n.generalTimeAgo}';
    }
  }

  /// Formats the time difference between [dateTime] and now into a human-readable string.
  /// The difference can be either positive (future date) or negative (past date).
  static String prettyPrintTimeDifference(AppLocalizations l10n, DateTime dateTime) {
    final difference = dateTime.difference(clock.now()).abs();
    String time = '';
    if (difference.inDays >= 1) {
      time = l10n.generalDays(difference.inDays);
    } else if (difference.inHours >= 1) {
      time = l10n.generalHours(difference.inHours);
    } else if (difference.inMinutes >= 1) {
      time = l10n.generalMinutes(difference.inMinutes);
    } else {
      time = l10n.generalTimeAgoLessThenOneMinute;
    }
    return time;
  }
}
