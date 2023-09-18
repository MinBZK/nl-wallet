import 'package:equatable/equatable.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

@JsonSerializable()
sealed class DataValue extends Equatable {
  const DataValue();

  /// Temporary method to not break mock data + UI at this point
  String stringValue();
}

class DataValueString extends DataValue {
  final String value;

  const DataValueString(this.value);

  @override
  List<Object> get props => [value];

  @override
  String stringValue() => value;
}

class DataValueInteger extends DataValue {
  final int value;

  const DataValueInteger(this.value);

  @override
  List<Object> get props => [value];

  @override
  String stringValue() => value.toString();
}

class DataValueDouble extends DataValue {
  final double value;

  const DataValueDouble(this.value);

  @override
  List<Object> get props => [value];

  @override
  String stringValue() => value.toString();
}

class DataValueBoolean extends DataValue {
  final bool value;

  const DataValueBoolean(this.value);

  @override
  List<Object> get props => [value];

  @override
  String stringValue() => value.toString();
}

class DataValueDate extends DataValue {
  final String value;

  const DataValueDate(this.value);

  @override
  List<Object> get props => [value];

  @override
  String stringValue() => value.toString();
}
