// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'home_screen_argument.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

HomeScreenArgument _$HomeScreenArgumentFromJson(Map<String, dynamic> json) => HomeScreenArgument(
      cards: (json['cards'] as List<dynamic>).map((e) => WalletCard.fromJson(e as Map<String, dynamic>)).toList(),
    );

Map<String, dynamic> _$HomeScreenArgumentToJson(HomeScreenArgument instance) => <String, dynamic>{
      'cards': instance.cards.map((e) => e.toJson()).toList(),
    };
