import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/page/wallet_transferring_page.dart';

class WalletTransferTargetTransferringPage extends StatelessWidget {
  final VoidCallback onStopPressed;

  const WalletTransferTargetTransferringPage({required this.onStopPressed, super.key});

  @override
  Widget build(BuildContext context) {
    return WalletTransferringPage(
      title: context.l10n.walletTransferTargetScreenTransferringTitle,
      description: context.l10n.walletTransferTargetScreenTransferringDescription,
      cta: context.l10n.generalStop,
      onCtaPressed: onStopPressed,
    );
  }
}
