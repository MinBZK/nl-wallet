// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'wallet_card.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_WalletCard _$WalletCardFromJson(Map<String, dynamic> json) => _WalletCard(
  attestationId: json['attestationId'] as String?,
  attestationType: json['attestationType'] as String,
  issuer: Organization.fromJson(json['issuer'] as Map<String, dynamic>),
  status: $enumDecode(_$CardStatusEnumMap, json['status']),
  validFrom: DateTime.parse(json['validFrom'] as String),
  validUntil: DateTime.parse(json['validUntil'] as String),
  attributes: (json['attributes'] as List<dynamic>)
      .map((e) => DataAttribute.fromJson(e as Map<String, dynamic>))
      .toList(),
  metadata:
      (json['metadata'] as List<dynamic>?)
          ?.map((e) => CardDisplayMetadata.fromJson(e as Map<String, dynamic>))
          .toList() ??
      const [],
);

Map<String, dynamic> _$WalletCardToJson(_WalletCard instance) => <String, dynamic>{
  'attestationId': instance.attestationId,
  'attestationType': instance.attestationType,
  'issuer': instance.issuer.toJson(),
  'status': _$CardStatusEnumMap[instance.status]!,
  'validFrom': instance.validFrom.toIso8601String(),
  'validUntil': instance.validUntil.toIso8601String(),
  'attributes': instance.attributes.map((e) => e.toJson()).toList(),
  'metadata': instance.metadata.map((e) => e.toJson()).toList(),
};

const _$CardStatusEnumMap = {
  CardStatus.validSoon: 'validSoon',
  CardStatus.valid: 'valid',
  CardStatus.expiresSoon: 'expiresSoon',
  CardStatus.expired: 'expired',
  CardStatus.revoked: 'revoked',
  CardStatus.corrupted: 'corrupted',
  CardStatus.unknown: 'unknown',
};
