import 'package:flutter/material.dart';

import '../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../domain/model/timeline/signing_timeline_attribute.dart';
import '../../domain/model/timeline/timeline_attribute.dart';

class TimelineAttributeStatusColorMapper {
  static Color map(ThemeData themeData, TimelineAttribute attribute) {
    if (attribute is InteractionTimelineAttribute) return InteractionStatusColorMapper.map(themeData, attribute.status);
    if (attribute is OperationTimelineAttribute) return themeData.colorScheme.onBackground;
    if (attribute is SigningTimelineAttribute) return themeData.colorScheme.onBackground;
    throw ('Unsupported attribute: $attribute');
  }
}

class InteractionStatusColorMapper {
  static Color map(ThemeData themeData, InteractionStatus status) {
    switch (status) {
      case InteractionStatus.failed:
        return themeData.colorScheme.error;
      default:
        return themeData.colorScheme.onBackground;
    }
  }
}
