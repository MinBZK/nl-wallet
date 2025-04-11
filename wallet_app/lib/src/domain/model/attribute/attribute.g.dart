// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'attribute.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

DataAttribute _$DataAttributeFromJson(Map<String, dynamic> json) => DataAttribute(
      key: json['key'] as String,
      label: const LocalizedTextConverter().fromJson(json['label'] as Map<String, dynamic>),
      value: const AttributeValueConverter().fromJson(json['value'] as Map<String, dynamic>),
      sourceCardDocType: json['sourceCardDocType'] as String,
    );

Map<String, dynamic> _$DataAttributeToJson(DataAttribute instance) => <String, dynamic>{
      'key': instance.key,
      'label': const LocalizedTextConverter().toJson(instance.label),
      'value': const AttributeValueConverter().toJson(instance.value),
      'sourceCardDocType': instance.sourceCardDocType,
    };
