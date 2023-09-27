import 'dart:ui';

import 'package:collection/collection.dart';

import '../../../../bridge_generated.dart';
import 'attribute/card_attribute_value_mapper.dart';

class CardSubtitleMapper {
  final CardAttributeValueMapper _valueMapper;

  CardSubtitleMapper(this._valueMapper);

  String map(Card card, Locale locale) {
    switch (card.docType) {
      case 'pid_id':
      case 'com.example.pid':
        final nameAttribute =
            card.attributes.firstWhereOrNull((attribute) => attribute.key.toLowerCase().contains('name'));
        if (nameAttribute == null) return '';
        return _valueMapper.map(nameAttribute.value, locale);
      case 'pid_address':
      case 'com.example.address':
        final cityAttribute =
            card.attributes.firstWhereOrNull((attribute) => attribute.key.toLowerCase().contains('city'));
        if (cityAttribute == null) return '';
        return _valueMapper.map(cityAttribute.value, locale);
      default:
        return '';
    }
  }
}
