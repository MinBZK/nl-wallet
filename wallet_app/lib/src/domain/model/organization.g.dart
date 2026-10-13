// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'organization.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_Organization _$OrganizationFromJson(Map<String, dynamic> json) => _Organization(
  id: json['id'] as String,
  legalName: json['legalName'] as String,
  displayName: json['displayName'] as String,
  publicBody: json['publicBody'] as bool?,
  personType: $enumDecodeNullable(_$PersonTypeEnumMap, json['personType']),
  description: _$JsonConverterFromJson<Map<String, dynamic>, Map<Locale, String>>(
    json['description'],
    const LocalizedTextConverter().fromJson,
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
  'publicBody': instance.publicBody,
  'personType': _$PersonTypeEnumMap[instance.personType],
  'description': _$JsonConverterToJson<Map<String, dynamic>, Map<Locale, String>>(
    instance.description,
    const LocalizedTextConverter().toJson,
  ),
  'webUri': instance.webUri,
  'supportUri': instance.supportUri,
  'privacyPolicyUri': instance.privacyPolicyUri,
  'countryCode': instance.countryCode,
  'organizationId': instance.organizationId,
};

const _$PersonTypeEnumMap = {
  PersonType.legalPerson: 'legalPerson',
  PersonType.naturalPerson: 'naturalPerson',
};

Value? _$JsonConverterFromJson<Json, Value>(
  Object? json,
  Value? Function(Json json) fromJson,
) => json == null ? null : fromJson(json as Json);

Json? _$JsonConverterToJson<Json, Value>(
  Value? value,
  Json? Function(Value value) toJson,
) => value == null ? null : toJson(value);
