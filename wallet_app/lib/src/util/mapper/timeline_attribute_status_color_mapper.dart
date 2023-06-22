import 'package:flutter/material.dart';

import '../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../domain/model/timeline/signing_timeline_attribute.dart';
import '../../domain/model/timeline/timeline_attribute.dart';
import '../extension/build_context_extension.dart';

class TimelineAttributeStatusColorMapper {
  static Color map(BuildContext context, TimelineAttribute attribute) {
    if (attribute is InteractionTimelineAttribute) return InteractionStatusColorMapper.map(context, attribute.status);
    if (attribute is OperationTimelineAttribute) return context.colorScheme.onBackground;
    if (attribute is SigningTimelineAttribute) return context.colorScheme.onBackground;
    throw ('Unsupported attribute: $attribute');
  }
}

class InteractionStatusColorMapper {
  static Color map(BuildContext context, InteractionStatus status) {
    switch (status) {
      case InteractionStatus.success:
        return context.colorScheme.onBackground;
      case InteractionStatus.failed:
        return context.colorScheme.error;
      case InteractionStatus.rejected:
        return context.colorScheme.error;
    }
  }
}
