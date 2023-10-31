import 'package:equatable/equatable.dart';

import 'value/gender.dart';

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

  const BooleanValue(this.value);
}

class DateValue extends AttributeValue {
  @override
  final DateTime value;

  const DateValue(this.value);
}

class GenderValue extends AttributeValue {
  @override
  final Gender value;

  const GenderValue(this.value);
}
