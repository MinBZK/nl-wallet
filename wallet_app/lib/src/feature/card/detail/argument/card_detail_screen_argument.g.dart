// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'card_detail_screen_argument.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

CardDetailScreenArgument _$CardDetailScreenArgumentFromJson(Map<String, dynamic> json) => CardDetailScreenArgument(
      card: json['card'] == null ? null : WalletCard.fromJson(json['card'] as Map<String, dynamic>),
      cardId: json['cardId'] as String,
      cardTitle: Map<String, String>.from(json['cardTitle'] as Map),
    );

Map<String, dynamic> _$CardDetailScreenArgumentToJson(CardDetailScreenArgument instance) => <String, dynamic>{
      'card': instance.card?.toJson(),
      'cardId': instance.cardId,
      'cardTitle': instance.cardTitle,
    };
