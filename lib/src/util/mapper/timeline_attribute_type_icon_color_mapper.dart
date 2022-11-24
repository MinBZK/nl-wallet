import 'package:flutter/material.dart';

import '../../domain/model/timeline_attribute.dart';

const _kNeutralDarkBlueColor = Color(0xFF0D193B);

class TimelineAttributeTypeIconColorMapper {
  static Color map(ThemeData themeData, TimelineAttribute attribute) {
    if (attribute is InteractionAttribute) return InteractionTypeColorMapper.map(themeData, attribute.interactionType);
    if (attribute is OperationAttribute) return _kNeutralDarkBlueColor;
    throw ('Unsupported attribute: $attribute');
  }
}

class InteractionTypeColorMapper {
  static Color map(ThemeData themeData, InteractionType type) {
    switch (type) {
      case InteractionType.success:
        return themeData.primaryColor;
      case InteractionType.rejected:
        return _kNeutralDarkBlueColor;
      case InteractionType.failed:
        return themeData.errorColor;
    }
  }
}
