import 'package:equatable/equatable.dart';

import 'attribute_type.dart';
import 'attribute_value_type.dart';

export 'attribute_type.dart';
export 'attribute_value_type.dart';

abstract class Attribute extends Equatable {
  final AttributeType type;
  final AttributeValueType valueType;

  const Attribute({required this.type, required this.valueType});
}
