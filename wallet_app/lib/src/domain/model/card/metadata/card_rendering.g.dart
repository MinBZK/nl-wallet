// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'card_rendering.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

SimpleCardRendering _$SimpleCardRenderingFromJson(Map<String, dynamic> json) => SimpleCardRendering(
      logo: _$JsonConverterFromJson<Map<String, dynamic>, AppImageData>(
          json['logo'], const AppImageDataConverter().fromJson),
      logoAltText: json['logoAltText'] as String?,
      bgColor: _$JsonConverterFromJson<int, Color>(json['bgColor'], const ColorConverter().fromJson),
      textColor: _$JsonConverterFromJson<int, Color>(json['textColor'], const ColorConverter().fromJson),
    );

Map<String, dynamic> _$SimpleCardRenderingToJson(SimpleCardRendering instance) => <String, dynamic>{
      'logo': _$JsonConverterToJson<Map<String, dynamic>, AppImageData>(
          instance.logo, const AppImageDataConverter().toJson),
      'logoAltText': instance.logoAltText,
      'bgColor': _$JsonConverterToJson<int, Color>(instance.bgColor, const ColorConverter().toJson),
      'textColor': _$JsonConverterToJson<int, Color>(instance.textColor, const ColorConverter().toJson),
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
