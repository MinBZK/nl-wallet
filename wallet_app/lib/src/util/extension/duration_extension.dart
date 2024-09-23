extension DurationExtension on Duration {
  /// Returns the (+/-) number of months in this [Duration].
  int get inMonths => inDays ~/ 30;
}
