// coverage:ignore-file
import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/button/confirm/confirm_buttons.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/secondary_button.dart';

class DigidConfirmButtons extends StatelessWidget {
  final VoidCallback? onAccept;
  final VoidCallback? onDecline;

  const DigidConfirmButtons({this.onAccept, this.onDecline, super.key});

  @override
  Widget build(BuildContext context) {
    return ConfirmButtons(
      primaryButton: PrimaryButton(
        key: const Key('acceptButton'),
        onPressed: onAccept,
        text: Text(context.l10n.mockDigidScreenAcceptCta),
        icon: null,
      ),
      secondaryButton: SecondaryButton(
        key: const Key('rejectButton'),
        onPressed: onDecline,
        text: Text(context.l10n.mockDigidScreenDeclineCta),
        icon: const Icon(Icons.block_flipped),
      ),
    );
  }
}
