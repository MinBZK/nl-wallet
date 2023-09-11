import 'package:flutter/material.dart';

import 'attribute.dart';

export 'attribute_value_type.dart';

class UiAttribute extends Attribute {
  final String value;
  final IconData icon;

  const UiAttribute({
    required this.value,
    required this.icon,
    super.key = '',
    required super.label,
    super.valueType = AttributeValueType.text,
  });

  @override
  String get key => throw UnsupportedError('UiAttributes should only be used to render attributes to the screen');

  @override
  List<Object?> get props => [value, icon, key, label];
}
