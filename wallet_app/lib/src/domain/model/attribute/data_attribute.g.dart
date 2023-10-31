// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'data_attribute.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

DataAttribute _$DataAttributeFromJson(Map<String, dynamic> json) => DataAttribute(
      key: json['key'] as String,
      label: Map<String, String>.from(json['label'] as Map),
      value: _$JsonConverterFromJson<Map<String, dynamic>, AttributeValue>(
          json['value'], const AttributeValueConverter().fromJson),
      sourceCardId: json['sourceCardId'] as String,
    );

Map<String, dynamic> _$DataAttributeToJson(DataAttribute instance) => <String, dynamic>{
      'key': instance.key,
      'label': instance.label,
      'value': const AttributeValueConverter().toJson(instance.value),
      'sourceCardId': instance.sourceCardId,
    };

Value? _$JsonConverterFromJson<Json, Value>(
  Object? json,
  Value? Function(Json json) fromJson,
) =>
    json == null ? null : fromJson(json as Json);
