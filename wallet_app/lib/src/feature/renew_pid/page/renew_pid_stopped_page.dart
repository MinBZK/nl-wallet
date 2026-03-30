import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/terminal_page.dart';
import '../../common/widget/button/primary_button.dart';
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
      primaryButton: PrimaryButton(
        text: Text(context.l10n.renewPidStoppedCloseCta),
        icon: const Icon(Icons.close_outlined),
        onPressed: onPrimaryPressed,
        key: const Key('primaryButtonCta'),
      ),
    );
  }
}
