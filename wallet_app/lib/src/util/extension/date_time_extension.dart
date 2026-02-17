import 'package:clock/clock.dart';

extension DateTimeExtension on DateTime {
  /// Returns year & month (resets all other date/time data)
  DateTime get yearMonth {
    if (isUtc) {
      return DateTime.utc(year, month);
    } else {
      return DateTime(year, month);
    }
  }

  bool get isToday {
    final now = clock.now();
    return now.year == year && now.month == month && now.day == day;
  }

  bool get isInLastWeek {
    final oneWeekAgo = clock.now().subtract(const Duration(days: 7));
    return isAfter(oneWeekAgo);
  }

  bool get isInLastMonth {
    final oneMonthAgo = clock.now().subtract(const Duration(days: 31));
    return isAfter(oneMonthAgo);
  }

  /// Returns true when `other` date is on the exact same day (ignoring time)
  bool isSameDay(DateTime other) {
    return year == other.year && month == other.month && day == other.day;
  }
}
