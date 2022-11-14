extension DateTimeExtension on DateTime {
  /// Returns year & month only (resets all other date/time data)
  DateTime yearMonthOnly() => DateTime(year, month);
}
