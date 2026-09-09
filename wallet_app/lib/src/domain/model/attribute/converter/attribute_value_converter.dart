import 'dart:convert';

import 'package:json_annotation/json_annotation.dart';

import '../../converter/app_image_data_converter.dart';
import '../attribute_value.dart';

/// Keys used in the json structure
const _kTypeKey = 'type';
const _kValueKey = 'value';

/// Values used to associate to the correct AttributeValue type
const _kStringValue = 'string';
const _kBooleanValue = 'bool';
const _kNumberValue = 'number';
const _kDateValue = 'date';
const _kArrayValue = 'array';
const _kBytesValue = 'bytes';
const _kImageValue = 'image';
const _kMapValue = 'map';
const _kNullValue = 'null';

class AttributeValueConverter extends JsonConverter<AttributeValue, Map<String, dynamic>> {
  const AttributeValueConverter();

  @override
  AttributeValue fromJson(Map<String, dynamic> json) {
    switch (json[_kTypeKey]) {
      case _kStringValue:
        return StringValue(json[_kValueKey]!);
      case _kBooleanValue:
        return BooleanValue(bool.parse(json[_kValueKey]!));
      case _kNumberValue:
        return NumberValue(num.parse(json[_kValueKey]!));
      case _kDateValue:
        return DateValue(_decodeDateTime(json[_kValueKey]!));
      case _kArrayValue:
        return ArrayValue(List<AttributeValue>.from(json[_kValueKey].map(fromJson)));
      case _kBytesValue:
        return BytesValue(const Base64Decoder().convert(json[_kValueKey]!));
      case _kImageValue:
        return ImageValue(const AppImageDataConverter().fromJson(json[_kValueKey]!));
      case _kMapValue:
        return MapValue(
          (json[_kValueKey] as Map<String, dynamic>).map((key, value) => MapEntry(key, fromJson(value))),
        );
      case _kNullValue:
        return NullValue();
    }
    throw UnsupportedError('Unknown type: ${json[_kTypeKey]}');
  }

  @override
  Map<String, dynamic> toJson(AttributeValue object) {
    switch (object) {
      case StringValue():
        return {_kTypeKey: _kStringValue, _kValueKey: object.value};
      case BooleanValue():
        return {_kTypeKey: _kBooleanValue, _kValueKey: object.value.toString()};
      case NumberValue():
        return {_kTypeKey: _kNumberValue, _kValueKey: object.value.toString()};
      case DateValue():
        return {_kTypeKey: _kDateValue, _kValueKey: _encodeDateTime(object.value)};
      case ArrayValue():
        return {_kTypeKey: _kArrayValue, _kValueKey: object.value.map(toJson).toList()};
      case BytesValue():
        return {_kTypeKey: _kBytesValue, _kValueKey: const Base64Encoder().convert(object.value)};
      case ImageValue():
        return {_kTypeKey: _kImageValue, _kValueKey: const AppImageDataConverter().toJson(object.value)};
      case MapValue():
        return {_kTypeKey: _kMapValue, _kValueKey: object.value.map((key, value) => MapEntry(key, toJson(value)))};
      case NullValue():
        return {_kTypeKey: _kNullValue, _kValueKey: null};
    }
  }

  int _encodeDateTime(DateTime dateTime) => dateTime.millisecondsSinceEpoch;

  DateTime _decodeDateTime(int millisecondsSinceEpoch) => DateTime.fromMillisecondsSinceEpoch(millisecondsSinceEpoch);
}
