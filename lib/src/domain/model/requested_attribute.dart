import 'data_attribute.dart';

class RequestedAttribute {
  final String name;
  final DataAttributeType type;
  final DataAttributeValueType valueType;

  const RequestedAttribute({
    required this.name,
    required this.type,
    required this.valueType,
  });
}
