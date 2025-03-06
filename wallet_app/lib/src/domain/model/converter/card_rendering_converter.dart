import 'package:json_annotation/json_annotation.dart';

import '../card/metadata/card_rendering.dart';

class CardRenderingConverter extends JsonConverter<CardRendering, Map<String, dynamic>> {
  const CardRenderingConverter();

  @override
  CardRendering fromJson(Map<String, dynamic> json) {
    // Add SvgCardRendering and logic to discern between the two when we start supporting SVGs.
    return SimpleCardRendering.fromJson(json);
  }

  @override
  Map<String, dynamic> toJson(CardRendering object) {
    switch (object) {
      case SimpleCardRendering():
        return object.toJson();
    }
  }
}
