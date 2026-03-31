import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/terminal_page.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/tertiary_button.dart';
import '../../common/widget/page_illustration.dart';

class RenewPidLoginCancelledPage extends StatelessWidget {
  final VoidCallback onPrimaryPressed;
  final VoidCallback onSecondaryButtonPressed;

  const RenewPidLoginCancelledPage({
    required this.onPrimaryPressed,
    required this.onSecondaryButtonPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TerminalPage(
      title: context.l10n.renewPidDigidLoginCancelledTitle,
      description: context.l10n.renewPidDigidLoginCancelledDescription,
      illustration: const PageIllustration(asset: WalletAssets.svg_stopped),
      primaryButton: PrimaryButton(
        text: Text(context.l10n.renewPidDigidLoginCancelledRetryCta),
        icon: Image.asset(WalletAssets.logo_digid),
        onPressed: onPrimaryPressed,
        key: const Key('primaryButtonCta'),
      ),
      secondaryButton: TertiaryButton(
        text: Text(context.l10n.renewPidDigidLoginCancelledOpenWebsiteCta),
        icon: const Icon(Icons.north_east_outlined),
        onPressed: onSecondaryButtonPressed,
        key: const Key('secondaryButtonCta'),
      ),
    );
  }
}
