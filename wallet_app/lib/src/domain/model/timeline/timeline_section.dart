import 'timeline_attribute.dart';

class TimelineSection {
  final DateTime dateTime;
  final List<TimelineAttribute> attributes;

  const TimelineSection(this.dateTime, this.attributes);
}
