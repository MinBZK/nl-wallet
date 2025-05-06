import 'dart:ui';

import 'package:equatable/equatable.dart';
import 'package:json_annotation/json_annotation.dart';

import '../../app_image_data.dart';
import '../../converter/app_image_data_converter.dart';
import '../../converter/color_converter.dart';

part 'card_rendering.g.dart';

sealed class CardRendering extends Equatable {
  const CardRendering();
}

@JsonSerializable(converters: [ColorConverter(), AppImageDataConverter()], explicitToJson: true)
class SimpleCardRendering extends CardRendering {
  final AppImageData? logo;
  final String? logoAltText;
  final Color? bgColor;
  final Color? textColor;

  const SimpleCardRendering({this.logo, this.logoAltText, this.bgColor, this.textColor});

  factory SimpleCardRendering.fromJson(Map<String, dynamic> json) => _$SimpleCardRenderingFromJson(json);

  Map<String, dynamic> toJson() => _$SimpleCardRenderingToJson(this);

  @override
  List<Object?> get props => [logo, logoAltText, bgColor, textColor];
}
