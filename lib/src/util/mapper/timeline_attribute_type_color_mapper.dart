import 'package:flutter/material.dart';

import '../../domain/model/timeline_attribute.dart';

class TimelineAttributeTypeColorMapper {
  static Color map(ThemeData themeData, TimelineAttribute attribute) {
    if (attribute is InteractionAttribute) return InteractionTypeColorMapper.map(themeData, attribute.interactionType);
    if (attribute is OperationAttribute) return themeData.primaryColorDark;
    throw ('Unsupported attribute: $attribute');
  }
}

class InteractionTypeColorMapper {
  static Color map(ThemeData themeData, InteractionType type) {
    switch (type) {
      case InteractionType.success:
        return themeData.primaryColor;
      default:
        return themeData.primaryColorDark;
    }
  }
}
