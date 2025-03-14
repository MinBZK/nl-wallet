import 'package:equatable/equatable.dart';

sealed class AttributeValue extends Equatable {
  /// Dynamic value getter, used to implement [Equatable] once vs in every subclass
  dynamic get value;

  const AttributeValue();

  @override
  List<Object?> get props => [value];

  @override
  String toString() => value.toString();
}

class StringValue extends AttributeValue {
  @override
  final String value;

  const StringValue(this.value);
}

class BooleanValue extends AttributeValue {
  @override
  final bool value;

  //ignore: avoid_positional_boolean_parameters
  const BooleanValue(this.value);
}

class NumberValue extends AttributeValue {
  @override
  final int value;

  const NumberValue(this.value);
}

class DateValue extends AttributeValue {
  @override
  final DateTime value;

  const DateValue(this.value);
}
