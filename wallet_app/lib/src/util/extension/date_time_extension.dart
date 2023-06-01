extension DateTimeExtension on DateTime {
  /// Returns year & month (resets all other date/time data)
  DateTime get yearMonth {
    if (isUtc) {
      return DateTime.utc(year, month);
    } else {
      return DateTime(year, month);
    }
  }
}
