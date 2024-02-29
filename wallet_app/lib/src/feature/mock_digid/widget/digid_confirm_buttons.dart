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
      onPrimaryPressed: onAccept ?? () {},
      onSecondaryPressed: onDecline ?? () {},
      primaryText: context.l10n.mockDigidScreenAcceptCta,
      secondaryText: context.l10n.mockDigidScreenDeclineCta,
    );
  }
}
