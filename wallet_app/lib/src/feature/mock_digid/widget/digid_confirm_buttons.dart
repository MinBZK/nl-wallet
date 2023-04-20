import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/button/confirm_buttons.dart';

class DigidConfirmButtons extends StatelessWidget {
  final VoidCallback? onAccept;
  final VoidCallback? onDecline;

  const DigidConfirmButtons({this.onAccept, this.onDecline, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return ConfirmButtons(
      onAccept: onAccept ?? () {},
      onDecline: onDecline ?? () {},
      acceptText: locale.mockDigidScreenAcceptCta,
      declineText: locale.mockDigidScreenDeclineCta,
    );
  }
}
