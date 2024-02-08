import 'package:flutter/cupertino.dart';

import '../../domain/model/attribute/attribute.dart';
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
  static String format(BuildContext context, TimelineAttribute attribute, {bool showOperationTitle = true}) {
    switch (attribute) {
      case InteractionTimelineAttribute():
        return attribute.organization.displayName.l10nValue(context);
      case OperationTimelineAttribute():
        return showOperationTitle ? attribute.card.front.title.l10nValue(context) : '';
      case SigningTimelineAttribute():
        return attribute.organization.displayName.l10nValue(context);
    }
    throw ('Unsupported attribute: $attribute');
  }
}
