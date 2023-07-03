import 'package:flutter/material.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import 'pin_page.dart';

class PinScreen extends StatelessWidget {
  final VoidCallback? onUnlock;

  const PinScreen({this.onUnlock, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(context.l10n.pinScreenTitle),
        leading: const SizedBox.shrink(),
        actions: [
          IconButton(
            onPressed: () => Navigator.of(context).restorablePushNamed(WalletRoutes.aboutRoute),
            icon: const Icon(Icons.info_outline),
            tooltip: context.l10n.pinScreenAboutAppTooltip,
          ),
        ],
      ),
      body: PinPage(onPinValidated: onUnlock),
    );
  }
}
