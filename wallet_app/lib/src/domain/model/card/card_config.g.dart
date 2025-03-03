// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'card_config.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

CardConfig _$CardConfigFromJson(Map<String, dynamic> json) => CardConfig(
      updatable: json['updatable'] as bool? ?? false,
      removable: json['removable'] as bool? ?? false,
    );

Map<String, dynamic> _$CardConfigToJson(CardConfig instance) => <String, dynamic>{
      'updatable': instance.updatable,
      'removable': instance.removable,
    };
