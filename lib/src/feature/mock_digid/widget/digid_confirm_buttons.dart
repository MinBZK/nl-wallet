import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/confirm_buttons.dart';

class DigidConfirmButtons extends StatelessWidget {
  final VoidCallback? onAccept;

  const DigidConfirmButtons({this.onAccept, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return ConfirmButtons(
      onAccept: onAccept ?? () {},
      onDecline: () {},
      acceptText: locale.mockDigidScreenAcceptCta,
      declineText: locale.mockDigidScreenDeclineCta,
    );
  }
}
