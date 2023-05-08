extension DurationExtension on Duration {
  int get inMonths => inDays ~/ 30;
}
