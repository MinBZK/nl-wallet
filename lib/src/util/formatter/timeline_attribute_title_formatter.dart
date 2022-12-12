import '../../domain/model/timeline_attribute.dart';

class TimelineAttributeTitleFormatter {
  /// Formats the title for [attribute].
  ///
  /// Returns [attribute] `cardTitle` when [attribute] `is` [OperationAttribute] and
  /// [showOperationTitle] == true.
  ///
  /// When [TimelineAttribute] is displayed in for example a single card specific context; set:
  /// [showOperationTitle]: false
  static String format(TimelineAttribute attribute, {bool showOperationTitle = true}) {
    if (attribute is InteractionAttribute) return attribute.organization.shortName;
    if (attribute is OperationAttribute) return showOperationTitle ? attribute.cardTitle : '';
    throw ('Unsupported attribute: $attribute');
  }
}
