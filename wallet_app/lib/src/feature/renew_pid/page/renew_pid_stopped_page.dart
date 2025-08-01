import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/terminal_page.dart';
import '../../common/widget/page_illustration.dart';

class RenewPidStoppedPage extends StatelessWidget {
  final VoidCallback onPrimaryPressed;

  const RenewPidStoppedPage({
    required this.onPrimaryPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TerminalPage(
      title: context.l10n.renewPidStoppedTitle,
      description: context.l10n.renewPidStoppedDescription,
      illustration: const PageIllustration(asset: WalletAssets.svg_stopped),
      primaryButtonCta: context.l10n.renewPidStoppedCloseCta,
      primaryButtonIcon: const Icon(Icons.close_outlined),
      onPrimaryPressed: onPrimaryPressed,
    );
  }
}
