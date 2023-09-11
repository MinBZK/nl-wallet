import 'attribute.dart';

class RequestedAttribute extends Attribute {
  const RequestedAttribute({required super.key, required super.label, required super.valueType});

  @override
  List<Object?> get props => [key, label, valueType];
}
