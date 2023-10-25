import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/card_front.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../wallet_core/wallet_core.dart';
import '../locale_mapper.dart';

/// Maps a [RequestedCard] to a [WalletCard] and enriches with (currently) hardcoded data.
class RequestedCardMapper extends LocaleMapper<RequestedCard, WalletCard> {
  final LocaleMapper<Card, CardFront> _cardFrontMapper;
  final LocaleMapper<CardAttribute, DataAttribute> _attributeMapper;

  RequestedCardMapper(this._cardFrontMapper, this._attributeMapper);

  @override
  WalletCard map(Locale locale, RequestedCard input) {
    const String cardId = ''; // FIXME: Do we need an actual ID here?
    return WalletCard(
      id: cardId,
      issuerId: '', // FIXME: Eventually remove issuerId (mock builds still rely on them for now)
      front: _cardFrontMapper.map(
        locale,
        Card(persistence: const CardPersistence.inMemory(), docType: input.docType, attributes: input.attributes),
      ), // Use a inMemory placeholder card to map to the (still mocked) CardFront
      attributes: input.attributes.map((attribute) => _attributeMapper.map(locale, attribute)).toList(),
    );
  }
}
