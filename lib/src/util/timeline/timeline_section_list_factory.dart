import '../../domain/model/timeline_attribute.dart';
import '../../domain/model/timeline_section.dart';
import '../extension/date_time_extension.dart';

class TimelineSectionListFactory {
  static List<TimelineSection> create(List<TimelineAttribute> attributes) {
    final Map<DateTime, List<TimelineAttribute>> monthYearMap = _monthYearAttributeMap(attributes);
    return monthYearMap.entries.map((e) => TimelineSection(e.key, e.value)).toList();
  }

  static Map<DateTime, List<TimelineAttribute>> _monthYearAttributeMap(List<TimelineAttribute> attributes) {
    Map<DateTime, List<TimelineAttribute>> map = {};

    for (TimelineAttribute attribute in attributes) {
      final DateTime yearMonthKey = attribute.dateTime.yearMonthOnly();

      List<TimelineAttribute>? mapEntry = map[yearMonthKey];
      if (mapEntry != null) {
        mapEntry.add(attribute);
      } else {
        map[yearMonthKey] = [attribute];
      }
    }
    return map;
  }
}
