import '../../../domain/model/card_front.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../wallet_assets.dart';
import '../../../wallet_core/wallet_core.dart';
import 'card_attribute_mapper.dart';

/// Maps a [Card] to a [WalletCard] and enriches with (currently) hardcoded data.
class CardMapper {
  final CardAttributeMapper _attributeMapper;

  CardMapper(this._attributeMapper);

  WalletCard map(Card input, String languageCode) {
    final String cardId = input.persistence.map(
      inMemory: (inMemory) => '',
      stored: (stored) => stored.id,
    );
    return WalletCard(
      id: cardId,
      issuerId: '', // FIXME: Eventually remove issuerId (mock builds still rely on them for now)
      front: _getCardFront(input.docType, languageCode),
      attributes: input.attributes.map((attribute) => _attributeMapper.map(attribute, languageCode)).toList(),
    );
  }

  CardFront _getCardFront(String docType, String languageCode) {
    switch (docType) {
      case 'pid_id':
        return CardFront(
          title: languageCode == 'nl' ? 'Persoonsgegevens' : 'Personal data',
          subtitle: 'Willeke',
          logoImage: WalletAssets.logo_card_rijksoverheid,
          holoImage: WalletAssets.svg_rijks_card_holo,
          backgroundImage: WalletAssets.svg_rijks_card_bg_light,
          theme: CardFrontTheme.light,
        );
      case 'pid_address':
        return CardFront(
          title: languageCode == 'nl' ? 'Woonadres' : 'Residential address',
          subtitle: languageCode == 'nl' ? 's-Gravenhage' : 'The Hague',
          logoImage: WalletAssets.logo_card_rijksoverheid,
          holoImage: WalletAssets.svg_rijks_card_holo,
          backgroundImage: WalletAssets.svg_rijks_card_bg_dark,
          theme: CardFrontTheme.dark,
        );
    }
    throw Exception('Unknown docType: $docType');
  }
}
