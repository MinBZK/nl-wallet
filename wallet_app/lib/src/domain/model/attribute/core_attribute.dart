import 'attribute.dart';

export 'attribute_value_type.dart';

class CoreAttribute extends Attribute {
  final dynamic rawValue;

  const CoreAttribute({
    required super.key,
    required super.label,
    required this.rawValue,
    super.valueType = AttributeValueType.text,
  });

  String get value => rawValue.toString();

  @override
  List<Object?> get props => [key, valueType, label, value];
}
