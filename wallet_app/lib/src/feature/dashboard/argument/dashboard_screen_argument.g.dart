// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'dashboard_screen_argument.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

DashboardScreenArgument _$DashboardScreenArgumentFromJson(
  Map<String, dynamic> json,
) => DashboardScreenArgument(
  cards: (json['cards'] as List<dynamic>).map((e) => WalletCard.fromJson(e as Map<String, dynamic>)).toList(),
);

Map<String, dynamic> _$DashboardScreenArgumentToJson(
  DashboardScreenArgument instance,
) => <String, dynamic>{'cards': instance.cards.map((e) => e.toJson()).toList()};
