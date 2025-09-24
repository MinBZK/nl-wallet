import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/page/wallet_transferring_page.dart';

class WalletTransferSourceTransferringPage extends StatelessWidget {
  final VoidCallback onStopPressed;

  const WalletTransferSourceTransferringPage({required this.onStopPressed, super.key});

  @override
  Widget build(BuildContext context) {
    return WalletTransferringPage(
      title: context.l10n.walletTransferSourceScreenTransferringTitle,
      description: context.l10n.walletTransferSourceScreenTransferringDescription,
      cta: context.l10n.generalStop,
      onCtaPressed: onStopPressed,
    );
  }
}
