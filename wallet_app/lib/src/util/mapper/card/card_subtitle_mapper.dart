import 'package:collection/collection.dart';

import '../../../wallet_core/wallet_core.dart';
import '../locale_mapper.dart';

class CardSubtitleMapper extends LocaleMapper<Card, String> {
  final LocaleMapper<CardValue, String> _attributeValueMapper;

  CardSubtitleMapper(this._attributeValueMapper);

  @override
  String map(Locale locale, Card input) {
    switch (input.docType) {
      case 'pid_id':
      case 'com.example.pid':
        final nameAttribute =
            input.attributes.firstWhereOrNull((attribute) => attribute.key.toLowerCase().contains('name'));
        if (nameAttribute == null) return '';
        return _attributeValueMapper.map(locale, nameAttribute.value);
      case 'pid_address':
      case 'com.example.address':
        final cityAttribute =
            input.attributes.firstWhereOrNull((attribute) => attribute.key.toLowerCase().contains('city'));
        if (cityAttribute == null) return '';
        return _attributeValueMapper.map(locale, cityAttribute.value);
      default:
        return '';
    }
  }
}
