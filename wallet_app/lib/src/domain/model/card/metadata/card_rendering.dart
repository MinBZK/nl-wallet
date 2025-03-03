import 'dart:ui';

import 'package:equatable/equatable.dart';
import 'package:json_annotation/json_annotation.dart';

import '../../converter/color_converter.dart';

part 'card_rendering.g.dart';

sealed class CardRendering extends Equatable {
  const CardRendering();
}

@JsonSerializable(converters: [ColorConverter()], explicitToJson: true)
class SimpleCardRendering extends CardRendering {
  final String? logoUri;
  final String? logoAltText;
  final Color? bgColor;
  final Color? textColor;

  const SimpleCardRendering({this.logoUri, this.logoAltText, this.bgColor, this.textColor});

  factory SimpleCardRendering.fromJson(Map<String, dynamic> json) => _$SimpleCardRenderingFromJson(json);

  Map<String, dynamic> toJson() => _$SimpleCardRenderingToJson(this);

  @override
  List<Object?> get props => [logoUri, logoAltText, bgColor, textColor];
}
