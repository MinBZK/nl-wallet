import 'package:flutter/material.dart';

import '../widget/digid_confirm_buttons.dart';
import '../widget/digid_sign_in_with_header.dart';
import '../widget/digid_sign_in_with_organization.dart';

class DigidConfirmAppPage extends StatelessWidget {
  final VoidCallback onDeclinePressed;
  final VoidCallback onConfirmPressed;

  const DigidConfirmAppPage({
    required this.onConfirmPressed,
    required this.onDeclinePressed,
    Key? key,
  }) : super(key: key);

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
            const DigidSignInWithOrganization(),
            const Spacer(),
            DigidConfirmButtons(
              onAccept: onConfirmPressed,
              onDecline: onDeclinePressed,
            ),
          ],
        ),
      ),
    );
  }
}
