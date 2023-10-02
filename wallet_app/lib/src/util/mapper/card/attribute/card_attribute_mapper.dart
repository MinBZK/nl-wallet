import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../wallet_core/wallet_core.dart';
import '../../locale_mapper.dart';

class CardAttributeMapper extends LocaleMapper<CardAttribute, DataAttribute> {
  final LocaleMapper<List<LocalizedString>, String> _attributeLabelMapper;
  final LocaleMapper<CardValue, String> _attributeValueMapper;

  CardAttributeMapper(this._attributeLabelMapper, this._attributeValueMapper);

  @override
  DataAttribute map(Locale locale, CardAttribute input) {
    return DataAttribute(
      key: input.key,
      label: _attributeLabelMapper.map(locale, input.labels),
      value: _attributeValueMapper.map(locale, input.value),
      sourceCardId: '',
      valueType: AttributeValueType.text,
    );
  }
}
