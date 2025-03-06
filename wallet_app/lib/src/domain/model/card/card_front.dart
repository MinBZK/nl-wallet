import 'package:equatable/equatable.dart';
import 'package:json_annotation/json_annotation.dart';

import '../attribute/converter/localized_string_converter.dart';
import '../localized_text.dart';

part 'card_front.g.dart';

@JsonSerializable(converters: [LocalizedStringConverter()], explicitToJson: true)
class CardFront extends Equatable {
  final LocalizedText title;
  final LocalizedText? subtitle;
  final LocalizedText? info;
  final String? logoImage;
  final String? holoImage;
  final String backgroundImage;
  final CardFrontTheme theme;

  const CardFront({
    required this.title,
    this.subtitle,
    this.info,
    this.logoImage,
    this.holoImage,
    required this.backgroundImage,
    required this.theme,
  });

  factory CardFront.fromJson(Map<String, dynamic> json) => _$CardFrontFromJson(json);

  Map<String, dynamic> toJson() => _$CardFrontToJson(this);

  @override
  List<Object?> get props => [title, subtitle, info, logoImage, holoImage, backgroundImage, theme];
}

enum CardFrontTheme {
  light, // light background + dark texts
  dark, // dark background + light texts
}
