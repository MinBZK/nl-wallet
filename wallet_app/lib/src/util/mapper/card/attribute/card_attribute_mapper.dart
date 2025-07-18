import 'package:equatable/equatable.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/attribute/attribute.dart';
import '../../mapper.dart';

class CardAttributeMapper extends Mapper<CardAttributeWithCardId, DataAttribute> {
  final Mapper<core.AttributeValue, AttributeValue> _attributeValueMapper;
  final Mapper<List<core.ClaimDisplayMetadata>, LocalizedText> _localizedLabelsMapper;

  CardAttributeMapper(this._attributeValueMapper, this._localizedLabelsMapper);

  @override
  DataAttribute map(CardAttributeWithCardId input) {
    return DataAttribute(
      key: input.attribute.key,
      svgId: input.attribute.svgId,
      label: _localizedLabelsMapper.map(input.attribute.labels),
      value: _attributeValueMapper.map(input.attribute.value),
    );
  }
}

class CardAttributeWithCardId extends Equatable {
  final String? cardId;
  final core.AttestationAttribute attribute;

  const CardAttributeWithCardId(this.cardId, this.attribute);

  @override
  List<Object?> get props => [cardId, attribute];
}
