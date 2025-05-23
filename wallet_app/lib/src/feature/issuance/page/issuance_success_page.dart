import 'package:flutter/material.dart';

import '../../../domain/model/card/wallet_card.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/terminal_page.dart';
import '../../common/widget/page_illustration.dart';

class IssuanceSuccessPage extends StatelessWidget {
  final VoidCallback onClose;
  final List<WalletCard> cards;

  const IssuanceSuccessPage({
    required this.onClose,
    required this.cards,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TerminalPage(
      title: context.l10n.issuanceSuccessPageTitle(cards.length),
      description: context.l10n.issuanceSuccessPageCardsAddedSubtitle(cards.length),
      primaryButtonCta: context.l10n.issuanceSuccessPageCloseCta,
      primaryButtonIcon: Icon(Icons.arrow_forward_outlined),
      onPrimaryPressed: onClose,
      illustration: PageIllustration(asset: WalletAssets.svg_phone),
    );
  }
}
