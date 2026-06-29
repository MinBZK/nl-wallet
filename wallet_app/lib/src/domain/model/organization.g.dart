// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'organization.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_Organization _$OrganizationFromJson(Map<String, dynamic> json) => _Organization(
  id: json['id'] as String,
  legalName: json['legalName'] as String,
  displayName: json['displayName'] as String,
  category: _$JsonConverterFromJson<Map<String, dynamic>, Map<Locale, String>>(
    json['category'],
    const LocalizedTextConverter().fromJson,
  ),
  description: _$JsonConverterFromJson<Map<String, dynamic>, Map<Locale, String>>(
    json['description'],
    const LocalizedTextConverter().fromJson,
  ),
  logo: const AppImageDataConverter().fromJson(
    json['logo'] as Map<String, dynamic>,
  ),
  webUrl: json['webUrl'] as String?,
  privacyPolicyUrl: json['privacyPolicyUrl'] as String?,
  countryCode: json['countryCode'] as String,
  city: _$JsonConverterFromJson<Map<String, dynamic>, Map<Locale, String>>(
    json['city'],
    const LocalizedTextConverter().fromJson,
  ),
  department: _$JsonConverterFromJson<Map<String, dynamic>, Map<Locale, String>>(
    json['department'],
    const LocalizedTextConverter().fromJson,
  ),
  organizationId: json['organizationId'] as String?,
);

Map<String, dynamic> _$OrganizationToJson(
  _Organization instance,
) => <String, dynamic>{
  'id': instance.id,
  'legalName': instance.legalName,
  'displayName': instance.displayName,
  'category': _$JsonConverterToJson<Map<String, dynamic>, Map<Locale, String>>(
    instance.category,
    const LocalizedTextConverter().toJson,
  ),
  'description': _$JsonConverterToJson<Map<String, dynamic>, Map<Locale, String>>(
    instance.description,
    const LocalizedTextConverter().toJson,
  ),
  'logo': const AppImageDataConverter().toJson(instance.logo),
  'webUrl': instance.webUrl,
  'privacyPolicyUrl': instance.privacyPolicyUrl,
  'countryCode': instance.countryCode,
  'city': _$JsonConverterToJson<Map<String, dynamic>, Map<Locale, String>>(
    instance.city,
    const LocalizedTextConverter().toJson,
  ),
  'department': _$JsonConverterToJson<Map<String, dynamic>, Map<Locale, String>>(
    instance.department,
    const LocalizedTextConverter().toJson,
  ),
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
