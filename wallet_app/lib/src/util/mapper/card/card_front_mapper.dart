import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/card_front.dart';
import '../../../wallet_assets.dart';
import '../../../wallet_core/wallet_core.dart';
import '../locale_mapper.dart';

class CardFrontMapper extends LocaleMapper<Card, CardFront> {
  final LocaleMapper<Card, String> _subtitleMapper;

  CardFrontMapper(this._subtitleMapper);

  @override
  CardFront map(Locale locale, Card input) {
    final l10n = lookupAppLocalizations(locale);
    switch (input.docType) {
      case 'pid_id':
      case 'com.example.pid':
        return CardFront(
          title: l10n.pidIdCardTitle,
          subtitle: _subtitleMapper.map(locale, input),
          logoImage: WalletAssets.logo_card_rijksoverheid,
          holoImage: WalletAssets.svg_rijks_card_holo,
          backgroundImage: WalletAssets.svg_rijks_card_bg_light,
          theme: CardFrontTheme.light,
        );
      case 'pid_address':
      case 'com.example.address':
        return CardFront(
          title: l10n.pidAddressCardTitle,
          subtitle: _subtitleMapper.map(locale, input),
          logoImage: WalletAssets.logo_card_rijksoverheid,
          holoImage: WalletAssets.svg_rijks_card_holo,
          backgroundImage: WalletAssets.svg_rijks_card_bg_dark,
          theme: CardFrontTheme.dark,
        );
    }
    throw Exception('Unknown docType: ${input.docType}');
  }
}
