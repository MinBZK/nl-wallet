import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/card_front.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../wallet_core/wallet_core.dart';
import '../locale_mapper.dart';

/// Maps a [Card] to a [WalletCard] and enriches with (currently) hardcoded data.
class CardMapper extends LocaleMapper<Card, WalletCard> {
  final LocaleMapper<Card, CardFront> _cardFrontMapper;
  final LocaleMapper<CardAttribute, DataAttribute> _attributeMapper;

  CardMapper(this._cardFrontMapper, this._attributeMapper);

  @override
  WalletCard map(Locale locale, Card input) {
    final String cardId = input.persistence.map(
      inMemory: (inMemory) => '',
      stored: (stored) => stored.id,
    );
    return WalletCard(
      id: cardId,
      issuerId: '', // FIXME: Eventually remove issuerId (mock builds still rely on them for now)
      front: _cardFrontMapper.map(locale, input),
      attributes: input.attributes.map((attribute) => _attributeMapper.map(locale, attribute)).toList(),
    );
  }
}
