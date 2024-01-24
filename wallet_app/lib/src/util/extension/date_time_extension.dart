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
    final now = DateTime.now();
    return now.year == year && now.month == month && now.day == day;
  }

  bool get isInLastWeek {
    final oneWeekAgo = DateTime.now().subtract(const Duration(days: 7));
    return isAfter(oneWeekAgo);
  }

  bool get isInLastMonth {
    final oneMonthAgo = DateTime.now().subtract(const Duration(days: 31));
    return isAfter(oneMonthAgo);
  }
}
