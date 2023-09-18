import '../../../domain/model/card_front.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../wallet_assets.dart';
import '../../../wallet_core/wallet_core.dart';
import 'card_attribute_mapper.dart';

/// Maps a [Card] to a [WalletCard] with (currently) hardcoded data to enrich the PID card
class CardMapper {
  final CardAttributeMapper _attributeMapper;

  CardMapper(this._attributeMapper);

  WalletCard map(Card input, String languageCode) {
    final String cardId = input.id.toString();
    return WalletCard(
      id: cardId,
      issuerId: input.issuer,
      front: CardFront(
        title: languageCode == 'nl' ? 'Persoonsgegevens' : 'Personal data',
        subtitle: 'Willeke',
        logoImage: WalletAssets.logo_card_rijksoverheid,
        holoImage: WalletAssets.svg_rijks_card_holo,
        backgroundImage: WalletAssets.svg_rijks_card_bg_light,
        theme: CardFrontTheme.light,
      ),
      attributes: input.attributes.map((attribute) => _attributeMapper.map(attribute, languageCode)).toList(),
    );
  }
}
