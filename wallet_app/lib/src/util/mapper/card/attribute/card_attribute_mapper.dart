import 'dart:ui';

import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../wallet_core/wallet_core.dart';
import 'card_attribute_label_mapper.dart';
import 'card_attribute_value_mapper.dart';

class CardAttributeMapper {
  final CardAttributeLabelMapper _labelMapper;
  final CardAttributeValueMapper _valueMapper;

  CardAttributeMapper(this._labelMapper, this._valueMapper);

  DataAttribute map(CardAttribute input, Locale locale) {
    return DataAttribute(
      key: input.key,
      label: _labelMapper.map(input.labels, locale.languageCode),
      value: _valueMapper.map(input.value, locale),
      sourceCardId: '',
      valueType: AttributeValueType.text,
    );
  }
}
