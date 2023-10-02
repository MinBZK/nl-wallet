import 'package:flutter/material.dart';

import '../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../../domain/model/timeline/signing_timeline_attribute.dart';
import '../../../domain/model/timeline/timeline_attribute.dart';
import '../mapper.dart';

class TimelineAttributeErrorStatusIconMapper extends Mapper<TimelineAttribute, IconData?> {
  final Mapper<InteractionStatus, IconData?> _interactionErrorStatusIconMapper;
  final Mapper<SigningStatus, IconData?> _signingErrorStatusIconMapper;

  TimelineAttributeErrorStatusIconMapper(this._interactionErrorStatusIconMapper, this._signingErrorStatusIconMapper);

  @override
  IconData? map(TimelineAttribute input) {
    if (input is InteractionTimelineAttribute) return _interactionErrorStatusIconMapper.map(input.status);
    if (input is OperationTimelineAttribute) return null;
    if (input is SigningTimelineAttribute) return _signingErrorStatusIconMapper.map(input.status);
    throw ('Unsupported attribute: $input');
  }
}
