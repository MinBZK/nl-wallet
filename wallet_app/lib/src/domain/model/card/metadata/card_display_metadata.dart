import 'dart:ui';

import 'package:equatable/equatable.dart';
import 'package:json_annotation/json_annotation.dart';

import '../../converter/card_rendering_converter.dart';
import '../../converter/locale_converter.dart';
import 'card_rendering.dart';

part 'card_display_metadata.g.dart';

@JsonSerializable(converters: [LocaleConverter(), CardRenderingConverter()], explicitToJson: true)
class CardDisplayMetadata extends Equatable {
  final Locale language;
  final String name;
  final String? description;
  final String? rawSummary;
  final CardRendering? rendering;

  const CardDisplayMetadata({
    required this.language,
    required this.name,
    this.description,
    this.rawSummary,
    this.rendering,
  });

  factory CardDisplayMetadata.fromJson(Map<String, dynamic> json) => _$CardDisplayMetadataFromJson(json);

  Map<String, dynamic> toJson() => _$CardDisplayMetadataToJson(this);

  @override
  List<Object?> get props => [language, name, description, rawSummary, rendering];
}
