// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'organization.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_Organization _$OrganizationFromJson(Map<String, dynamic> json) => _Organization(
  id: json['id'] as String,
  legalName: json['legalName'] as String,
  displayName: json['displayName'] as String,
  type: _$JsonConverterFromJson<Map<String, dynamic>, Map<Locale, String>>(
    json['type'],
    const LocalizedTextConverter().fromJson,
  ),
  description: _$JsonConverterFromJson<Map<String, dynamic>, Map<Locale, String>>(
    json['description'],
    const LocalizedTextConverter().fromJson,
  ),
  logo: _$JsonConverterFromJson<Map<String, dynamic>, AppImageData>(
    json['logo'],
    const AppImageDataConverter().fromJson,
  ),
  webUri: json['webUri'] as String?,
  supportUri: json['supportUri'] as String?,
  privacyPolicyUri: json['privacyPolicyUri'] as String?,
  countryCode: json['countryCode'] as String,
  organizationId: json['organizationId'] as String,
);

Map<String, dynamic> _$OrganizationToJson(_Organization instance) => <String, dynamic>{
  'id': instance.id,
  'legalName': instance.legalName,
  'displayName': instance.displayName,
  'type': _$JsonConverterToJson<Map<String, dynamic>, Map<Locale, String>>(
    instance.type,
    const LocalizedTextConverter().toJson,
  ),
  'description': _$JsonConverterToJson<Map<String, dynamic>, Map<Locale, String>>(
    instance.description,
    const LocalizedTextConverter().toJson,
  ),
  'logo': _$JsonConverterToJson<Map<String, dynamic>, AppImageData>(
    instance.logo,
    const AppImageDataConverter().toJson,
  ),
  'webUri': instance.webUri,
  'supportUri': instance.supportUri,
  'privacyPolicyUri': instance.privacyPolicyUri,
  'countryCode': instance.countryCode,
  'organizationId': instance.organizationId,
};

Value? _$JsonConverterFromJson<Json, Value>(
  Object? json,
  Value? Function(Json json) fromJson,
) => json == null ? null : fromJson(json as Json);

Json? _$JsonConverterToJson<Json, Value>(
  Value? value,
  Json? Function(Value value) toJson,
) => value == null ? null : toJson(value);
