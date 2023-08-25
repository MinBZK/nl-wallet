// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'card_front.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

CardFront _$CardFrontFromJson(Map<String, dynamic> json) => CardFront(
      title: json['title'] as String,
      subtitle: json['subtitle'] as String?,
      info: json['info'] as String?,
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
