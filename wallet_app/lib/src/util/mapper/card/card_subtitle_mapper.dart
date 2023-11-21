import 'package:collection/collection.dart';
import 'package:wallet_core/core.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../formatter/attribute_value_formatter.dart';
import '../mapper.dart';

class CardSubtitleMapper extends Mapper<Card, LocalizedText?> {
  final Mapper<CardValue, AttributeValue> _attributeValueMapper;

  CardSubtitleMapper(this._attributeValueMapper);

  @override
  LocalizedText? map(Card input) {
    switch (input.docType) {
      case 'pid_id':
      case 'com.example.pid':
        final nameAttribute =
            input.attributes.firstWhereOrNull((attribute) => attribute.key.toLowerCase().contains('name'));
        if (nameAttribute == null) return null;
        return AppLocalizations.supportedLocales.asMap().map(
          (_, locale) {
            final attributeValue = _attributeValueMapper.map(nameAttribute.value);
            final formattedValue = AttributeValueFormatter.formatWithLocale(locale, attributeValue);
            return MapEntry(locale.languageCode, formattedValue);
          },
        );
      case 'pid_address':
      case 'com.example.address':
        final cityAttribute =
            input.attributes.firstWhereOrNull((attribute) => attribute.key.toLowerCase().contains('city'));
        if (cityAttribute == null) return null;
        return AppLocalizations.supportedLocales.asMap().map(
          (_, locale) {
            final attributeValue = _attributeValueMapper.map(cityAttribute.value);
            final formattedValue = AttributeValueFormatter.formatWithLocale(locale, attributeValue);
            return MapEntry(locale.languageCode, formattedValue);
          },
        );
      default:
        return null;
    }
  }
}
