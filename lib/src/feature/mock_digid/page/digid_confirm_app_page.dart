import 'package:flutter/material.dart';

import '../widget/digid_confirm_buttons.dart';
import '../widget/digid_sign_in_with_header.dart';
import '../widget/digid_sign_in_with_wallet.dart';

class DigidConfirmAppPage extends StatelessWidget {
  final VoidCallback onConfirmClicked;

  const DigidConfirmAppPage({required this.onConfirmClicked, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.max,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Align(
              alignment: Alignment.centerRight,
              child: Icon(Icons.close),
            ),
            const DigidSignInWithHeader(),
            const Spacer(),
            const Center(child: DigidSignInWithWallet()),
            const Spacer(),
            DigidConfirmButtons(onAccept: onConfirmClicked),
          ],
        ),
      ),
    );
  }
}
