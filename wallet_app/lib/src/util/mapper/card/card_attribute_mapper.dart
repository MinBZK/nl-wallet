import '../../../domain/model/attribute/data_attribute.dart';
import '../../../wallet_core/wallet_core.dart';
import 'card_attribute_label_mapper.dart';

class CardAttributeMapper {
  final CardAttributeLabelMapper _labelMapper;

  CardAttributeMapper(this._labelMapper);

  DataAttribute map(CardAttribute input, String languageCode) {
    return DataAttribute(
      key: input.key,
      label: _labelMapper.map(input.labels, languageCode),
      value: input.value.toString(),
      sourceCardId: '-',
      valueType: AttributeValueType.text,
    );
  }
}
