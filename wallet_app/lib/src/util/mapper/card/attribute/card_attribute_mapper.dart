import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../wallet_core/wallet_core.dart';
import '../../mapper.dart';

class CardAttributeMapper extends Mapper<CardAttribute, DataAttribute> {
  final Mapper<CardValue, AttributeValue> _attributeValueMapper;
  final Mapper<List<LocalizedString>, LocalizedText> _localizedLabelsMapper;

  CardAttributeMapper(this._attributeValueMapper, this._localizedLabelsMapper);

  @override
  DataAttribute map(CardAttribute input) {
    return DataAttribute(
      key: input.key,
      label: _localizedLabelsMapper.map(input.labels),
      value: _attributeValueMapper.map(input.value),
      sourceCardId: '',
    );
  }
}
