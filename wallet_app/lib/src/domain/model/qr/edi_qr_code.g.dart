// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'edi_qr_code.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

EdiQrCode _$EdiQrCodeFromJson(Map<String, dynamic> json) => EdiQrCode(
  id: json['id'] as String,
  type: $enumDecode(_$EdiQrTypeEnumMap, json['type']),
);

Map<String, dynamic> _$EdiQrCodeToJson(EdiQrCode instance) => <String, dynamic>{
  'id': instance.id,
  'type': _$EdiQrTypeEnumMap[instance.type]!,
};

const _$EdiQrTypeEnumMap = {
  EdiQrType.issuance: 'issue',
  EdiQrType.disclosure: 'verify',
  EdiQrType.sign: 'sign',
};
