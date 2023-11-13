import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/card_front.dart';
import '../../../domain/model/localized_text.dart';
import '../../../wallet_assets.dart';
import '../../../wallet_core/wallet_core.dart';
import '../mapper.dart';

class CardFrontMapper extends Mapper<Card, CardFront> {
  final Mapper<Card, LocalizedText?> _subtitleMapper;

  CardFrontMapper(this._subtitleMapper);

  @override
  CardFront map(Card input) {
    final l10ns = AppLocalizations.supportedLocales.map((e) => lookupAppLocalizations(e)).toList();
    switch (input.docType) {
      case 'pid_id':
      case 'com.example.pid':
        return CardFront(
          title: l10ns.asMap().map((_, l10n) => MapEntry(l10n.localeName, l10n.pidIdCardTitle)),
          subtitle: _subtitleMapper.map(input),
          logoImage: WalletAssets.logo_card_rijksoverheid,
          holoImage: WalletAssets.svg_rijks_card_holo,
          backgroundImage: WalletAssets.svg_rijks_card_bg_light,
          theme: CardFrontTheme.light,
        );
      case 'pid_address':
      case 'com.example.address':
        return CardFront(
          title: l10ns.asMap().map((_, l10n) => MapEntry(l10n.localeName, l10n.pidAddressCardTitle)),
          subtitle: _subtitleMapper.map(input),
          logoImage: WalletAssets.logo_card_rijksoverheid,
          holoImage: WalletAssets.svg_rijks_card_holo,
          backgroundImage: WalletAssets.svg_rijks_card_bg_dark,
          theme: CardFrontTheme.dark,
        );
    }
    throw Exception('Unknown docType: ${input.docType}');
  }
}
