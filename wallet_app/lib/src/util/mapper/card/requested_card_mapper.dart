import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/card_front.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../wallet_core/wallet_core.dart';
import '../mapper.dart';

/// Maps a [RequestedCard] to a [WalletCard] and enriches with (currently) hardcoded data.
class RequestedCardMapper extends Mapper<RequestedCard, WalletCard> {
  final Mapper<Card, CardFront> _cardFrontMapper;
  final Mapper<CardAttribute, DataAttribute> _attributeMapper;

  RequestedCardMapper(this._cardFrontMapper, this._attributeMapper);

  @override
  WalletCard map(RequestedCard input) {
    const String cardId = ''; // FIXME: Do we need an actual ID here?
    return WalletCard(
      id: cardId,
      issuerId: '', // FIXME: Eventually remove issuerId (mock builds still rely on them for now)
      front: _cardFrontMapper.map(
        Card(persistence: const CardPersistence.inMemory(), docType: input.docType, attributes: input.attributes),
      ), // Use a inMemory placeholder card to map to the (still mocked) CardFront
      attributes: input.attributes.map((attribute) => _attributeMapper.map(attribute)).toList(),
    );
  }
}
