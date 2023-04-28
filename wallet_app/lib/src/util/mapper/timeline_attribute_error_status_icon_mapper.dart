import 'package:flutter/material.dart';

import '../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../domain/model/timeline/signing_timeline_attribute.dart';
import '../../domain/model/timeline/timeline_attribute.dart';

const _kErrorIconData = Icons.error_outline;

class TimelineAttributeErrorStatusIconMapper {
  static IconData? map(TimelineAttribute attribute) {
    if (attribute is InteractionTimelineAttribute) return InteractionErrorStatusIconMapper.map(attribute.status);
    if (attribute is OperationTimelineAttribute) return null;
    if (attribute is SigningTimelineAttribute) return SigningErrorStatusIconMapper.map(attribute.status);
    throw ('Unsupported attribute: $attribute');
  }
}

class InteractionErrorStatusIconMapper {
  static IconData? map(InteractionStatus status) {
    switch (status) {
      case InteractionStatus.rejected:
        return _kErrorIconData;
      case InteractionStatus.failed:
        return _kErrorIconData;
      default:
        return null;
    }
  }
}

class SigningErrorStatusIconMapper {
  static IconData? map(SigningStatus status) {
    switch (status) {
      case SigningStatus.rejected:
        return _kErrorIconData;
      default:
        return null;
    }
  }
}
