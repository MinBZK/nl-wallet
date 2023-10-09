import 'package:flutter/material.dart';

import '../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../../domain/model/timeline/signing_timeline_attribute.dart';
import '../../../domain/model/timeline/timeline_attribute.dart';
import '../../extension/build_context_extension.dart';
import '../context_mapper.dart';

class TimelineAttributeStatusColorMapper extends ContextMapper<TimelineAttribute, Color> {
  final ContextMapper<InteractionStatus, Color> _interactionStatusColorMapper;

  TimelineAttributeStatusColorMapper(this._interactionStatusColorMapper);

  @override
  Color map(BuildContext context, TimelineAttribute input) {
    if (input is InteractionTimelineAttribute) return _interactionStatusColorMapper.map(context, input.status);
    if (input is OperationTimelineAttribute) return context.colorScheme.onBackground;
    if (input is SigningTimelineAttribute) return context.colorScheme.onBackground;
    throw ('Unsupported attribute: $input');
  }
}
