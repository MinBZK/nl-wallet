import 'package:json_annotation/json_annotation.dart';

import '../attribute_value.dart';

/// Keys used in the json structure
const _kTypeKey = 'type';
const _kValueKey = 'value';

/// Values used to associate to the correct AttributeValue type
const _kStringValue = 'string';
const _kBooleanValue = 'bool';
const _kNumberValue = 'number';

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
        return NumberValue(BigInt.parse(json[_kValueKey]!));
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
    }
  }
}
