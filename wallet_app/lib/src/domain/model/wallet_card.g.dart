// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'wallet_card.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

WalletCard _$WalletCardFromJson(Map<String, dynamic> json) => WalletCard(
      id: json['id'] as String,
      docType: json['docType'] as String,
      front: CardFront.fromJson(json['front'] as Map<String, dynamic>),
      attributes:
          (json['attributes'] as List<dynamic>).map((e) => DataAttribute.fromJson(e as Map<String, dynamic>)).toList(),
      config: json['config'] == null ? const CardConfig() : CardConfig.fromJson(json['config'] as Map<String, dynamic>),
    );

Map<String, dynamic> _$WalletCardToJson(WalletCard instance) => <String, dynamic>{
      'id': instance.id,
      'docType': instance.docType,
      'front': instance.front.toJson(),
      'attributes': instance.attributes.map((e) => e.toJson()).toList(),
      'config': instance.config.toJson(),
    };
