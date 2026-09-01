import 'dart:typed_data';

import 'package:equatable/equatable.dart';

import '../app_image_data.dart';

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
  final num value;

  const NumberValue(this.value);
}

class DateValue extends AttributeValue {
  @override
  final DateTime value;

  const DateValue(this.value);
}

class ArrayValue extends AttributeValue {
  @override
  final List<AttributeValue> value;

  const ArrayValue(this.value);

  @override
  String toString() => value.join(', ');
}

/// Only provided by mdoc based attestations: bytes the core could not recognize as an image,
/// e.g. a JPEG 2000 portrait. Kept separate from [ImageValue] so that the UI can say the value
/// exists but can't be rendered, instead of pretending it was never provided.
class BytesValue extends AttributeValue {
  @override
  final Uint8List value;

  const BytesValue(this.value);

  @override
  String toString() => '<BYTES>';
}

/// Only provided by mdoc based attestations, e.g. the portrait of an mDL.
class ImageValue extends AttributeValue {
  @override
  final AppImageData value;

  const ImageValue(this.value);

  @override
  String toString() => '<IMAGE>';
}

/// Only provided by mdoc based attestations, e.g. a single entry of the mDL driving_privileges.
class MapValue extends AttributeValue {
  @override
  final Map<String, AttributeValue> value;

  const MapValue(this.value);

  @override
  String toString() => value.entries.map((it) => '${it.key}: ${it.value}').join(', ');
}

class NullValue extends AttributeValue {
  @override
  get value => null;

  @override
  String toString() => 'null';
}
