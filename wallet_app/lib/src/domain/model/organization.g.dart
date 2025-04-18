// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'organization.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

Organization _$OrganizationFromJson(Map<String, dynamic> json) => Organization(
      id: json['id'] as String,
      legalName: const LocalizedTextConverter().fromJson(json['legalName'] as Map<String, dynamic>),
      displayName: const LocalizedTextConverter().fromJson(json['displayName'] as Map<String, dynamic>),
      category: _$JsonConverterFromJson<Map<String, dynamic>, Map<Locale, String>>(
          json['category'], const LocalizedTextConverter().fromJson),
      description: _$JsonConverterFromJson<Map<String, dynamic>, Map<Locale, String>>(
          json['description'], const LocalizedTextConverter().fromJson),
      logo: const AppImageDataConverter().fromJson(json['logo'] as Map<String, dynamic>),
      webUrl: json['webUrl'] as String?,
      privacyPolicyUrl: json['privacyPolicyUrl'] as String?,
      countryCode: json['countryCode'] as String?,
      city: _$JsonConverterFromJson<Map<String, dynamic>, Map<Locale, String>>(
          json['city'], const LocalizedTextConverter().fromJson),
      department: _$JsonConverterFromJson<Map<String, dynamic>, Map<Locale, String>>(
          json['department'], const LocalizedTextConverter().fromJson),
      kvk: json['kvk'] as String?,
    );

Map<String, dynamic> _$OrganizationToJson(Organization instance) => <String, dynamic>{
      'id': instance.id,
      'legalName': const LocalizedTextConverter().toJson(instance.legalName),
      'displayName': const LocalizedTextConverter().toJson(instance.displayName),
      'category': _$JsonConverterToJson<Map<String, dynamic>, Map<Locale, String>>(
          instance.category, const LocalizedTextConverter().toJson),
      'description': _$JsonConverterToJson<Map<String, dynamic>, Map<Locale, String>>(
          instance.description, const LocalizedTextConverter().toJson),
      'logo': const AppImageDataConverter().toJson(instance.logo),
      'webUrl': instance.webUrl,
      'privacyPolicyUrl': instance.privacyPolicyUrl,
      'countryCode': instance.countryCode,
      'city': _$JsonConverterToJson<Map<String, dynamic>, Map<Locale, String>>(
          instance.city, const LocalizedTextConverter().toJson),
      'department': _$JsonConverterToJson<Map<String, dynamic>, Map<Locale, String>>(
          instance.department, const LocalizedTextConverter().toJson),
      'kvk': instance.kvk,
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
