import 'package:flutter/material.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../common/widget/wallet_app_bar.dart';
import 'pin_page.dart';

class PinScreen extends StatelessWidget {
  final OnPinValidatedCallback onUnlock;

  const PinScreen({required this.onUnlock, super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('pinScreen'),
      appBar: WalletAppBar(
        automaticallyImplyLeading: false,
        actions: [
          IconButton(
            onPressed: () => Navigator.of(context).restorablePushNamed(WalletRoutes.aboutRoute),
            icon: const Icon(Icons.help_outline_rounded),
            tooltip: context.l10n.pinScreenAboutAppTooltip,
          ),
        ],
      ),
      body: PinPage(
        onPinValidated: onUnlock,
        keyboardColor: context.colorScheme.primary,
        showTopDivider: true,
      ),
    );
  }
}
