import 'dart:ui';

import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/card_front.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../wallet_assets.dart';
import '../../../wallet_core/wallet_core.dart';
import 'attribute/card_attribute_mapper.dart';
import 'card_subtitle_mapper.dart';

/// Maps a [Card] to a [WalletCard] and enriches with (currently) hardcoded data.
class CardMapper {
  final CardAttributeMapper _attributeMapper;
  final CardSubtitleMapper _subtitleMapper;

  CardMapper(this._subtitleMapper, this._attributeMapper);

  WalletCard map(Card card, Locale locale) {
    final String cardId = card.persistence.map(
      inMemory: (inMemory) => '',
      stored: (stored) => stored.id,
    );
    return WalletCard(
      id: cardId,
      issuerId: '', // FIXME: Eventually remove issuerId (mock builds still rely on them for now)
      front: _getCardFront(card, locale),
      attributes: card.attributes.map((attribute) => _attributeMapper.map(attribute, locale)).toList(),
    );
  }

  CardFront _getCardFront(Card card, Locale locale) {
    final l10n = lookupAppLocalizations(locale);
    switch (card.docType) {
      case 'pid_id':
      case 'com.example.pid':
        return CardFront(
          title: l10n.pidIdCardTitle,
          subtitle: _subtitleMapper.map(card, locale),
          logoImage: WalletAssets.logo_card_rijksoverheid,
          holoImage: WalletAssets.svg_rijks_card_holo,
          backgroundImage: WalletAssets.svg_rijks_card_bg_light,
          theme: CardFrontTheme.light,
        );
      case 'pid_address':
      case 'com.example.address':
        return CardFront(
          title: l10n.pidAddressCardTitle,
          subtitle: _subtitleMapper.map(card, locale),
          logoImage: WalletAssets.logo_card_rijksoverheid,
          holoImage: WalletAssets.svg_rijks_card_holo,
          backgroundImage: WalletAssets.svg_rijks_card_bg_dark,
          theme: CardFrontTheme.dark,
        );
    }
    throw Exception('Unknown docType: ${card.docType}');
  }
}
