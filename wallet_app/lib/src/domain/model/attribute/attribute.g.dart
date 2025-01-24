// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'attribute.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

DataAttribute _$DataAttributeFromJson(Map<String, dynamic> json) => DataAttribute(
      key: json['key'] as String,
      label: Map<String, String>.from(json['label'] as Map),
      value: const AttributeValueConverter().fromJson(json['value'] as Map<String, dynamic>),
      sourceCardDocType: json['sourceCardDocType'] as String,
    );

Map<String, dynamic> _$DataAttributeToJson(DataAttribute instance) => <String, dynamic>{
      'key': instance.key,
      'label': instance.label,
      'value': const AttributeValueConverter().toJson(instance.value),
      'sourceCardDocType': instance.sourceCardDocType,
    };
