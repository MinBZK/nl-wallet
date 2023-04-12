import 'attribute.dart';

export 'attribute_type.dart';
export 'attribute_value_type.dart';

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

  @override
  List<Object?> get props => [valueType, label, value, type, sourceCardId];
}
