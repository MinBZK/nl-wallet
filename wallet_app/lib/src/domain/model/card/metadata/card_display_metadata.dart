import 'dart:ui';

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../converter/card_rendering_converter.dart';
import '../../converter/locale_converter.dart';
import 'card_rendering.dart';

part 'card_display_metadata.freezed.dart';
part 'card_display_metadata.g.dart';

@Freezed(copyWith: false)
abstract class CardDisplayMetadata with _$CardDisplayMetadata {
  const factory CardDisplayMetadata({
    @LocaleConverter() required Locale language,
    required String name,
    String? description,
    String? rawSummary,
    @CardRenderingConverter() CardRendering? rendering,
  }) = _CardDisplayMetadata;

  factory CardDisplayMetadata.fromJson(Map<String, dynamic> json) => _$CardDisplayMetadataFromJson(json);
}
