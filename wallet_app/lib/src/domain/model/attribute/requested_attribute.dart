import 'attribute.dart';

class RequestedAttribute extends Attribute {
  final String name;

  const RequestedAttribute({
    required this.name,
    required super.type,
    required super.valueType,
  });

  @override
  List<Object?> get props => [type, valueType, name];
}
