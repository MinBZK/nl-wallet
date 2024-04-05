import 'package:flutter/material.dart';

import '../../util/extension/build_context_extension.dart';
import '../common/widget/button/icon/info_icon_button.dart';
import '../common/widget/wallet_app_bar.dart';
import 'pin_page.dart';

class PinScreen extends StatelessWidget {
  final OnPinValidatedCallback onUnlock;

  const PinScreen({required this.onUnlock, super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('pinScreen'),
      appBar: const WalletAppBar(
        automaticallyImplyLeading: false,
        actions: [InfoIconButton()],
      ),
      body: PinPage(
        onPinValidated: onUnlock,
        keyboardColor: context.colorScheme.primary,
        showTopDivider: true,
      ),
    );
  }
}
