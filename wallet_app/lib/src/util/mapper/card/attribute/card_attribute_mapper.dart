import 'package:equatable/equatable.dart';
import 'package:wallet_core/core.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../mapper.dart';

class CardAttributeMapper extends Mapper<CardAttributeWithDocType, DataAttribute> {
  final Mapper<CardValue, AttributeValue> _attributeValueMapper;
  final Mapper<List<LocalizedString>, LocalizedText> _localizedLabelsMapper;

  CardAttributeMapper(this._attributeValueMapper, this._localizedLabelsMapper);

  @override
  DataAttribute map(CardAttributeWithDocType input) {
    return DataAttribute(
      key: input.attribute.key,
      label: _localizedLabelsMapper.map(input.attribute.labels),
      value: _attributeValueMapper.map(input.attribute.value),
      sourceCardDocType: input.docType,
    );
  }
}

class CardAttributeWithDocType extends Equatable {
  final String docType;
  final CardAttribute attribute;

  const CardAttributeWithDocType(this.docType, this.attribute);

  @override
  List<Object?> get props => [docType, attribute];
}
