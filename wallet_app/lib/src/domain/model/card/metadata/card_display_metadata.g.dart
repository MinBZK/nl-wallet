// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'card_display_metadata.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

CardDisplayMetadata _$CardDisplayMetadataFromJson(Map<String, dynamic> json) => CardDisplayMetadata(
      language: const LocaleConverter().fromJson(json['language'] as String),
      name: json['name'] as String,
      description: json['description'] as String?,
      rendering: _$JsonConverterFromJson<Map<String, dynamic>, CardRendering>(
          json['rendering'], const CardRenderingConverter().fromJson),
    );

Map<String, dynamic> _$CardDisplayMetadataToJson(CardDisplayMetadata instance) => <String, dynamic>{
      'language': const LocaleConverter().toJson(instance.language),
      'name': instance.name,
      'description': instance.description,
      'rendering': _$JsonConverterToJson<Map<String, dynamic>, CardRendering>(
          instance.rendering, const CardRenderingConverter().toJson),
    };

Value? _$JsonConverterFromJson<Json, Value>(
  Object? json,
  Value? Function(Json json) fromJson,
) =>
    json == null ? null : fromJson(json as Json);

Json? _$JsonConverterToJson<Json, Value>(
  Value? value,
  Json? Function(Value value) toJson,
) =>
    value == null ? null : toJson(value);
