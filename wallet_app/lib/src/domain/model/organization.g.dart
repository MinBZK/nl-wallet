// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'organization.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

Organization _$OrganizationFromJson(Map<String, dynamic> json) => Organization(
      id: json['id'] as String,
      legalName: Map<String, String>.from(json['legalName'] as Map),
      displayName: Map<String, String>.from(json['displayName'] as Map),
      category: (json['category'] as Map<String, dynamic>?)?.map(
        (k, e) => MapEntry(k, e as String),
      ),
      description: (json['description'] as Map<String, dynamic>?)?.map(
        (k, e) => MapEntry(k, e as String),
      ),
      logo: const AppImageDataConverter().fromJson(json['logo'] as Map<String, dynamic>),
      webUrl: json['webUrl'] as String?,
      privacyPolicyUrl: json['privacyPolicyUrl'] as String?,
      countryCode: json['countryCode'] as String?,
      city: (json['city'] as Map<String, dynamic>?)?.map(
        (k, e) => MapEntry(k, e as String),
      ),
      department: (json['department'] as Map<String, dynamic>?)?.map(
        (k, e) => MapEntry(k, e as String),
      ),
      kvk: json['kvk'] as String?,
    );

Map<String, dynamic> _$OrganizationToJson(Organization instance) => <String, dynamic>{
      'id': instance.id,
      'legalName': instance.legalName,
      'displayName': instance.displayName,
      'category': instance.category,
      'description': instance.description,
      'logo': const AppImageDataConverter().toJson(instance.logo),
      'webUrl': instance.webUrl,
      'privacyPolicyUrl': instance.privacyPolicyUrl,
      'countryCode': instance.countryCode,
      'city': instance.city,
      'department': instance.department,
      'kvk': instance.kvk,
    };
