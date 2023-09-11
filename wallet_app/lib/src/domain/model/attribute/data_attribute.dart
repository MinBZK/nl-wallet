import 'package:json_annotation/json_annotation.dart';

import 'attribute.dart';

export 'attribute_value_type.dart';

part 'data_attribute.g.dart';

@JsonSerializable()
class DataAttribute extends Attribute {
  final String value;
  final String sourceCardId;

  const DataAttribute({
    required super.key,
    required super.label,
    required this.value,
    required this.sourceCardId,
    required super.valueType,
  });

  factory DataAttribute.fromJson(Map<String, dynamic> json) => _$DataAttributeFromJson(json);

  Map<String, dynamic> toJson() => _$DataAttributeToJson(this);

  @override
  List<Object?> get props => [key, valueType, label, value, sourceCardId];
}
