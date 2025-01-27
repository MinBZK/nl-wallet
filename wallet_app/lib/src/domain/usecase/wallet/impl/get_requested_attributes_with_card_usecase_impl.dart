import 'package:collection/collection.dart';

import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../model/attribute/attribute.dart';
import '../../../model/wallet_card.dart';
import '../get_requested_attributes_with_card_usecase.dart';

class GetRequestedAttributesWithCardUseCaseImpl implements GetRequestedAttributesWithCardUseCase {
  final WalletCardRepository _repository;

  GetRequestedAttributesWithCardUseCaseImpl(this._repository);

  @override
  Future<Map<WalletCard, List<DataAttribute>>> invoke(List<MissingAttribute> requestedAttributes) async {
    final List<WalletCard> cards = await _repository.readAll();

    final List<DataAttribute> foundAttributes = [];
    for (final requestedAttr in requestedAttributes) {
      final dataAttr = await _findAttribute(cards, requestedAttr);
      if (dataAttr != null) foundAttributes.add(dataAttr);
    }

    return groupBy(foundAttributes, (attribute) {
      return _findCardById(cards, attribute.sourceCardDocType);
    });
  }

  Future<DataAttribute?> _findAttribute(List<WalletCard> cards, MissingAttribute requestedAttribute) async {
    for (final card in cards) {
      final foundAttribute = card.attributes.firstWhereOrNull((attr) => attr.key == requestedAttribute.key);
      if (foundAttribute != null) return foundAttribute;
    }
    return null;
  }

  WalletCard _findCardById(List<WalletCard> cards, String cardId) {
    return cards.firstWhere((card) => card.id == cardId);
  }
}
