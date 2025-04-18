// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'card_detail_screen_argument.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

CardDetailScreenArgument _$CardDetailScreenArgumentFromJson(Map<String, dynamic> json) => CardDetailScreenArgument(
      card: json['card'] == null ? null : WalletCard.fromJson(json['card'] as Map<String, dynamic>),
      cardId: json['cardId'] as String,
      cardTitle: const LocalizedTextConverter().fromJson(json['cardTitle'] as Map<String, dynamic>),
    );

Map<String, dynamic> _$CardDetailScreenArgumentToJson(CardDetailScreenArgument instance) => <String, dynamic>{
      'card': instance.card?.toJson(),
      'cardId': instance.cardId,
      'cardTitle': const LocalizedTextConverter().toJson(instance.cardTitle),
    };
