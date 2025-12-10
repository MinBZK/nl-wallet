// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'wallet_card.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_WalletCard _$WalletCardFromJson(Map<String, dynamic> json) => _WalletCard(
  attestationId: json['attestationId'] as String?,
  attestationType: json['attestationType'] as String,
  issuer: Organization.fromJson(json['issuer'] as Map<String, dynamic>),
  status: CardStatus.fromJson(json['status'] as Map<String, dynamic>),
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
  'status': instance.status.toJson(),
  'attributes': instance.attributes.map((e) => e.toJson()).toList(),
  'metadata': instance.metadata.map((e) => e.toJson()).toList(),
};
