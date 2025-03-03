// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'card_front.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

CardFront _$CardFrontFromJson(Map<String, dynamic> json) => CardFront(
      title: Map<String, String>.from(json['title'] as Map),
      subtitle: (json['subtitle'] as Map<String, dynamic>?)?.map(
        (k, e) => MapEntry(k, e as String),
      ),
      info: (json['info'] as Map<String, dynamic>?)?.map(
        (k, e) => MapEntry(k, e as String),
      ),
      logoImage: json['logoImage'] as String?,
      holoImage: json['holoImage'] as String?,
      backgroundImage: json['backgroundImage'] as String,
      theme: $enumDecode(_$CardFrontThemeEnumMap, json['theme']),
    );

Map<String, dynamic> _$CardFrontToJson(CardFront instance) => <String, dynamic>{
      'title': instance.title,
      'subtitle': instance.subtitle,
      'info': instance.info,
      'logoImage': instance.logoImage,
      'holoImage': instance.holoImage,
      'backgroundImage': instance.backgroundImage,
      'theme': _$CardFrontThemeEnumMap[instance.theme]!,
    };

const _$CardFrontThemeEnumMap = {
  CardFrontTheme.light: 'light',
  CardFrontTheme.dark: 'dark',
};
