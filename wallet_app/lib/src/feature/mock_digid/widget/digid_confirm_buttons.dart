import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/button/confirm/confirm_button.dart';
import '../../common/widget/button/confirm/confirm_buttons.dart';

class DigidConfirmButtons extends StatelessWidget {
  final VoidCallback? onAccept;
  final VoidCallback? onDecline;

  const DigidConfirmButtons({this.onAccept, this.onDecline, super.key});

  @override
  Widget build(BuildContext context) {
    return ConfirmButtons(
      primaryButton: ConfirmButton.accept(
        onPressed: onAccept,
        text: context.l10n.mockDigidScreenAcceptCta,
      ),
      secondaryButton: ConfirmButton.reject(
        onPressed: onDecline,
        text: context.l10n.mockDigidScreenDeclineCta,
        icon: Icons.block_flipped,
      ),
    );
  }
}
