import '../../domain/model/timeline_attribute.dart';

class TimelineAttributeTitleFormatter {
  static String format(TimelineAttribute attribute) {
    if (attribute is InteractionAttribute) return attribute.organization;
    if (attribute is OperationAttribute) return '';
    throw ('Unsupported attribute: $attribute');
  }
}
