import '../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../domain/model/timeline/signing_timeline_attribute.dart';
import '../../domain/model/timeline/timeline_attribute.dart';

class TimelineAttributeTitleFormatter {
  /// Formats the title for [attribute].
  ///
  /// Returns [attribute] `cardTitle` when [attribute] `is` [OperationTimelineAttribute] and
  /// [showOperationTitle] == true.
  ///
  /// When [TimelineAttribute] is displayed in for example a single card specific context; set:
  /// [showOperationTitle]: false
  static String format(TimelineAttribute attribute, {bool showOperationTitle = true}) {
    if (attribute is InteractionTimelineAttribute) return attribute.organization.shortName;
    if (attribute is OperationTimelineAttribute) return showOperationTitle ? attribute.cardTitle : '';
    if (attribute is SigningTimelineAttribute) return attribute.organization.shortName;
    throw ('Unsupported attribute: $attribute');
  }
}
