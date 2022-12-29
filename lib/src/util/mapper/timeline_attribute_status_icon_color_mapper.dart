import 'package:flutter/material.dart';

import '../../domain/model/timeline/timeline_attribute.dart';

const _kNeutralDarkBlueColor = Color(0xFF0D193B);

class TimelineAttributeStatusIconColorMapper {
  static Color map(ThemeData themeData, TimelineAttribute attribute) {
    if (attribute is InteractionAttribute) return InteractionStatusColorMapper.map(themeData, attribute.status);
    if (attribute is OperationAttribute) return _kNeutralDarkBlueColor;
    if (attribute is SigningAttribute) return SigningStatusColorMapper.map(themeData, attribute.status);
    throw ('Unsupported attribute: $attribute');
  }
}

class InteractionStatusColorMapper {
  static Color map(ThemeData themeData, InteractionStatus status) {
    switch (status) {
      case InteractionStatus.success:
        return themeData.primaryColor;
      case InteractionStatus.rejected:
        return _kNeutralDarkBlueColor;
      case InteractionStatus.failed:
        return themeData.errorColor;
    }
  }
}

class SigningStatusColorMapper {
  static Color map(ThemeData themeData, SigningStatus status) {
    switch (status) {
      case SigningStatus.success:
        return themeData.primaryColor;
      case SigningStatus.rejected:
        return _kNeutralDarkBlueColor;
    }
  }
}
