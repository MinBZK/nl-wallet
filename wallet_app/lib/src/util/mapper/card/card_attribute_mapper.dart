import '../../../domain/model/attribute/data_attribute.dart';
import '../../../wallet_core/wallet_core.dart';
import 'card_attribute_label_mapper.dart';
import 'card_value_mapper.dart';

class CardAttributeMapper {
  final CardAttributeLabelMapper _labelMapper;
  final CardValueMapper _valueMapper;

  CardAttributeMapper(this._labelMapper, this._valueMapper);

  DataAttribute map(CardAttribute input, String languageCode) {
    return DataAttribute(
      key: input.key,
      label: _labelMapper.map(input.labels, languageCode),
      value: _valueMapper.map(input.value).stringValue(),
      //TODO: PVW-1333 - Display translated labels
      sourceCardId: '-',
      valueType: AttributeValueType.text,
    );
  }
}
