import 'package:json_annotation/json_annotation.dart';

import 'attribute.dart';

export 'attribute_type.dart';
export 'attribute_value_type.dart';

part 'data_attribute.g.dart';

@JsonSerializable()
class DataAttribute extends Attribute {
  final String label;
  final String value;
  final String sourceCardId;

  const DataAttribute({
    required this.label,
    required this.value,
    required this.sourceCardId,
    super.type = AttributeType.other,
    required super.valueType,
  });

  factory DataAttribute.fromJson(Map<String, dynamic> json) => _$DataAttributeFromJson(json);

  Map<String, dynamic> toJson() => _$DataAttributeToJson(this);

  @override
  List<Object?> get props => [valueType, label, value, type, sourceCardId];
}
