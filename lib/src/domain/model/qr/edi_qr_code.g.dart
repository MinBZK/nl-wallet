// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'edi_qr_code.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

EdiQrCode _$EdiQrCodeFromJson(Map<String, dynamic> json) => EdiQrCode(
      id: json['id'] as int,
      type: $enumDecode(_$QrTypeEnumMap, json['type']),
    );

Map<String, dynamic> _$EdiQrCodeToJson(EdiQrCode instance) => <String, dynamic>{
      'id': instance.id,
      'type': _$QrTypeEnumMap[instance.type]!,
    };

const _$QrTypeEnumMap = {
  EdiQrType.issue: 'issue',
  EdiQrType.verify: 'verify',
};
