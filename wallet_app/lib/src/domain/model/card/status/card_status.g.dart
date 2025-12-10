// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'card_status.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

CardStatusValidSoon _$CardStatusValidSoonFromJson(Map<String, dynamic> json) => CardStatusValidSoon(
  validFrom: DateTime.parse(json['validFrom'] as String),
  $type: json['runtimeType'] as String?,
);

Map<String, dynamic> _$CardStatusValidSoonToJson(
  CardStatusValidSoon instance,
) => <String, dynamic>{
  'validFrom': instance.validFrom.toIso8601String(),
  'runtimeType': instance.$type,
};

CardStatusValid _$CardStatusValidFromJson(Map<String, dynamic> json) => CardStatusValid(
  validUntil: json['validUntil'] == null ? null : DateTime.parse(json['validUntil'] as String),
  $type: json['runtimeType'] as String?,
);

Map<String, dynamic> _$CardStatusValidToJson(CardStatusValid instance) => <String, dynamic>{
  'validUntil': instance.validUntil?.toIso8601String(),
  'runtimeType': instance.$type,
};

CardStatusExpiresSoon _$CardStatusExpiresSoonFromJson(
  Map<String, dynamic> json,
) => CardStatusExpiresSoon(
  validUntil: DateTime.parse(json['validUntil'] as String),
  $type: json['runtimeType'] as String?,
);

Map<String, dynamic> _$CardStatusExpiresSoonToJson(
  CardStatusExpiresSoon instance,
) => <String, dynamic>{
  'validUntil': instance.validUntil.toIso8601String(),
  'runtimeType': instance.$type,
};

CardStatusExpired _$CardStatusExpiredFromJson(Map<String, dynamic> json) => CardStatusExpired(
  validUntil: DateTime.parse(json['validUntil'] as String),
  $type: json['runtimeType'] as String?,
);

Map<String, dynamic> _$CardStatusExpiredToJson(CardStatusExpired instance) => <String, dynamic>{
  'validUntil': instance.validUntil.toIso8601String(),
  'runtimeType': instance.$type,
};

CardStatusRevoked _$CardStatusRevokedFromJson(Map<String, dynamic> json) =>
    CardStatusRevoked($type: json['runtimeType'] as String?);

Map<String, dynamic> _$CardStatusRevokedToJson(CardStatusRevoked instance) => <String, dynamic>{
  'runtimeType': instance.$type,
};

CardStatusCorrupted _$CardStatusCorruptedFromJson(Map<String, dynamic> json) =>
    CardStatusCorrupted($type: json['runtimeType'] as String?);

Map<String, dynamic> _$CardStatusCorruptedToJson(
  CardStatusCorrupted instance,
) => <String, dynamic>{'runtimeType': instance.$type};

CardStatusUndetermined _$CardStatusUndeterminedFromJson(
  Map<String, dynamic> json,
) => CardStatusUndetermined($type: json['runtimeType'] as String?);

Map<String, dynamic> _$CardStatusUndeterminedToJson(
  CardStatusUndetermined instance,
) => <String, dynamic>{'runtimeType': instance.$type};
