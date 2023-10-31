import 'package:json_annotation/json_annotation.dart';

import '../attribute_value.dart';
import '../value/gender.dart';

/// Keys used in the json structure
const _kTypeKey = 'type';
const _kValueKey = 'value';

/// Values used to associate to the correct AttributeValue type
const _kStringValue = 'string';
const _kBooleanValue = 'bool';
const _kDateValue = 'date';
const _kGenderValue = 'gender';

/// Map used to (consistently) convert the gender enum to a string
const _kGenderTypeEnumMap = {
  Gender.unknown: 'unknown',
  Gender.male: 'male',
  Gender.female: 'female',
  Gender.notApplicable: 'not_applicable',
};

class AttributeValueConverter extends JsonConverter<AttributeValue, Map<String, dynamic>> {
  const AttributeValueConverter();

  @override
  AttributeValue fromJson(Map<String, dynamic> json) {
    switch (json[_kTypeKey]) {
      case _kStringValue:
        return StringValue(json[_kValueKey]!);
      case _kBooleanValue:
        return BooleanValue(bool.parse(json[_kValueKey]!));
      case _kDateValue:
        return DateValue(_decodeDateTime(json[_kValueKey]!));
      case _kGenderValue:
        return GenderValue(_decodeGender(json[_kValueKey]!));
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
      case DateValue():
        return {_kTypeKey: _kDateValue, _kValueKey: _encodeDateTime(object.value)};
      case GenderValue():
        return {_kTypeKey: _kGenderValue, _kValueKey: _encodeGender(object.value)};
    }
  }

  String _encodeGender(Gender gender) => _kGenderTypeEnumMap[gender]!;

  Gender _decodeGender(String value) => _kGenderTypeEnumMap.entries.firstWhere((element) => element.value == value).key;

  int _encodeDateTime(DateTime dateTime) => dateTime.millisecondsSinceEpoch;

  DateTime _decodeDateTime(int millisecondsSinceEpoch) => DateTime.fromMillisecondsSinceEpoch(millisecondsSinceEpoch);
}
