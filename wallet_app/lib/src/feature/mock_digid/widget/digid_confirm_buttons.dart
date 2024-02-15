import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/button/confirm_buttons.dart';

class DigidConfirmButtons extends StatelessWidget {
  final VoidCallback? onAccept;
  final VoidCallback? onDecline;

  const DigidConfirmButtons({this.onAccept, this.onDecline, super.key});

  @override
  Widget build(BuildContext context) {
    return ConfirmButtons(
      onAcceptPressed: onAccept ?? () {},
      onDeclinePressed: onDecline ?? () {},
      acceptText: context.l10n.mockDigidScreenAcceptCta,
      declineText: context.l10n.mockDigidScreenDeclineCta,
    );
  }
}
