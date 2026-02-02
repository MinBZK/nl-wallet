import 'dart:ui';

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../app_image_data.dart';
import '../../converter/app_image_data_converter.dart';
import '../../converter/color_converter.dart';

part 'card_rendering.freezed.dart';
part 'card_rendering.g.dart';

@freezed
sealed class CardRendering with _$CardRendering {
  const factory CardRendering.simple({
    @AppImageDataConverter() AppImageData? logo,
    String? logoAltText,
    @ColorConverter() Color? bgColor,
    @ColorConverter() Color? textColor,
  }) = SimpleCardRendering;

  factory CardRendering.fromJson(Map<String, dynamic> json) => _$CardRenderingFromJson(json);
}
