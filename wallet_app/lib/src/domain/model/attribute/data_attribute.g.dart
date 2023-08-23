// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'data_attribute.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

DataAttribute _$DataAttributeFromJson(Map<String, dynamic> json) => DataAttribute(
      label: json['label'] as String,
      value: json['value'] as String,
      sourceCardId: json['sourceCardId'] as String,
      type: $enumDecodeNullable(_$AttributeTypeEnumMap, json['type']) ?? AttributeType.other,
      valueType: $enumDecode(_$AttributeValueTypeEnumMap, json['valueType']),
    );

Map<String, dynamic> _$DataAttributeToJson(DataAttribute instance) => <String, dynamic>{
      'type': _$AttributeTypeEnumMap[instance.type]!,
      'valueType': _$AttributeValueTypeEnumMap[instance.valueType]!,
      'label': instance.label,
      'value': instance.value,
      'sourceCardId': instance.sourceCardId,
    };

const _$AttributeTypeEnumMap = {
  AttributeType.firstNames: 'firstNames',
  AttributeType.lastName: 'lastName',
  AttributeType.birthName: 'birthName',
  AttributeType.fullName: 'fullName',
  AttributeType.gender: 'gender',
  AttributeType.profilePhoto: 'profilePhoto',
  AttributeType.birthDate: 'birthDate',
  AttributeType.birthPlace: 'birthPlace',
  AttributeType.birthCountry: 'birthCountry',
  AttributeType.citizenshipNumber: 'citizenshipNumber',
  AttributeType.nationality: 'nationality',
  AttributeType.documentNr: 'documentNr',
  AttributeType.issuanceDate: 'issuanceDate',
  AttributeType.expiryDate: 'expiryDate',
  AttributeType.healthInsuranceExpiryDate: 'healthInsuranceExpiryDate',
  AttributeType.height: 'height',
  AttributeType.university: 'university',
  AttributeType.education: 'education',
  AttributeType.educationLevel: 'educationLevel',
  AttributeType.certificateOfConduct: 'certificateOfConduct',
  AttributeType.phone: 'phone',
  AttributeType.email: 'email',
  AttributeType.postalCode: 'postalCode',
  AttributeType.streetName: 'streetName',
  AttributeType.houseNumber: 'houseNumber',
  AttributeType.city: 'city',
  AttributeType.olderThan18: 'olderThan18',
  AttributeType.healthIssuerId: 'healthIssuerId',
  AttributeType.healthIssuerClientId: 'healthIssuerClientId',
  AttributeType.drivingLicenseCategories: 'drivingLicenseCategories',
  AttributeType.other: 'other',
};

const _$AttributeValueTypeEnumMap = {
  AttributeValueType.image: 'image',
  AttributeValueType.text: 'text',
};
