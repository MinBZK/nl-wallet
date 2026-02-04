// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'card_detail_screen_argument.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_CardDetailScreenArgument _$CardDetailScreenArgumentFromJson(
  Map<String, dynamic> json,
) => _CardDetailScreenArgument(
  card: json['card'] == null ? null : WalletCard.fromJson(json['card'] as Map<String, dynamic>),
  cardId: json['cardId'] as String,
  cardTitle: _$JsonConverterFromJson<Map<String, dynamic>, Map<Locale, String>>(
    json['cardTitle'],
    const LocalizedTextConverter().fromJson,
  ),
);

Map<String, dynamic> _$CardDetailScreenArgumentToJson(
  _CardDetailScreenArgument instance,
) => <String, dynamic>{
  'card': instance.card?.toJson(),
  'cardId': instance.cardId,
  'cardTitle': _$JsonConverterToJson<Map<String, dynamic>, Map<Locale, String>>(
    instance.cardTitle,
    const LocalizedTextConverter().toJson,
  ),
};

Value? _$JsonConverterFromJson<Json, Value>(
  Object? json,
  Value? Function(Json json) fromJson,
) => json == null ? null : fromJson(json as Json);

Json? _$JsonConverterToJson<Json, Value>(
  Value? value,
  Json? Function(Value value) toJson,
) => value == null ? null : toJson(value);
