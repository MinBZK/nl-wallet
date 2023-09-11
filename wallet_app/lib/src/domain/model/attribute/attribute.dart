import 'package:equatable/equatable.dart';

import 'attribute_value_type.dart';

export 'attribute_value_type.dart';

abstract class Attribute extends Equatable {
  final AttributeKey key;
  final LocalizedLabel label;
  final AttributeValueType valueType;

  const Attribute({
    required this.key,
    required this.label,
    required this.valueType,
  });
}

typedef AttributeKey = String;
typedef LocalizedLabel = String;
