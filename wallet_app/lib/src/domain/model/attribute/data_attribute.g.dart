// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'data_attribute.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

DataAttribute _$DataAttributeFromJson(Map<String, dynamic> json) => DataAttribute(
      key: json['key'] as String,
      label: json['label'] as String,
      value: json['value'] as String,
      sourceCardId: json['sourceCardId'] as String,
      valueType: $enumDecode(_$AttributeValueTypeEnumMap, json['valueType']),
    );

Map<String, dynamic> _$DataAttributeToJson(DataAttribute instance) => <String, dynamic>{
      'key': instance.key,
      'label': instance.label,
      'valueType': _$AttributeValueTypeEnumMap[instance.valueType]!,
      'value': instance.value,
      'sourceCardId': instance.sourceCardId,
    };

const _$AttributeValueTypeEnumMap = {
  AttributeValueType.image: 'image',
  AttributeValueType.text: 'text',
};
