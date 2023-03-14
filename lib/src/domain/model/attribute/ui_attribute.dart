import 'package:flutter/material.dart';

import 'attribute.dart';

export 'attribute_type.dart';
export 'attribute_value_type.dart';

class UiAttribute extends Attribute {
  final String label;
  final String value;
  final IconData icon;

  const UiAttribute({
    required this.label,
    required this.value,
    required this.icon,
    super.type = AttributeType.other,
    super.valueType = AttributeValueType.text,
  });

  @override
  List<Object?> get props => [valueType, label, value, type, icon];
}
