import 'package:flutter/material.dart';

import '../../domain/model/timeline/timeline_attribute.dart';

class TimelineAttributeStatusIconMapper {
  static IconData map(TimelineAttribute attribute) {
    if (attribute is InteractionAttribute) return InteractionStatusIconMapper.map(attribute.status);
    if (attribute is OperationAttribute) return OperationStatusIconMapper.map(attribute.status);
    if (attribute is SigningAttribute) return SigningStatusIconMapper.map(attribute.status);
    throw ('Unsupported attribute: $attribute');
  }
}

class InteractionStatusIconMapper {
  static IconData map(InteractionStatus status) {
    switch (status) {
      case InteractionStatus.success:
        return Icons.check_outlined;
      case InteractionStatus.rejected:
        return Icons.not_interested_outlined;
      case InteractionStatus.failed:
        return Icons.priority_high_outlined;
    }
  }
}

class OperationStatusIconMapper {
  static IconData map(OperationStatus status) {
    return Icons.credit_card_outlined;
  }
}

class SigningStatusIconMapper {
  static IconData map(SigningStatus status) {
    switch (status) {
      case SigningStatus.success:
        return Icons.check_outlined;
      case SigningStatus.rejected:
        return Icons.not_interested_outlined;
    }
  }
}
