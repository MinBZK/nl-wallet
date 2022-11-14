import 'package:flutter/material.dart';

import '../../domain/model/timeline_attribute.dart';

class TimelineAttributeTypeIconMapper {
  static IconData map(TimelineAttribute attribute) {
    if (attribute is InteractionAttribute) return InteractionTypeIconMapper.map(attribute.interactionType);
    if (attribute is OperationAttribute) return OperationTypeIconMapper.map(attribute.operationType);
    throw ('Unsupported attribute: $attribute');
  }
}

class InteractionTypeIconMapper {
  static IconData map(InteractionType type) {
    switch (type) {
      case InteractionType.success:
        return Icons.check_outlined;
      case InteractionType.rejected:
        return Icons.not_interested_outlined;
      case InteractionType.failed:
        return Icons.priority_high_outlined;
    }
  }
}

class OperationTypeIconMapper {
  static IconData map(OperationType type) {
    return Icons.credit_card_outlined;
  }
}
