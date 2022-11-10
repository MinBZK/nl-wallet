import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/text_icon_button.dart';
import '../widget/status_icon.dart';

class VerificationDeclinedPage extends StatelessWidget {
  final VoidCallback? onHistoryPressed;
  final VoidCallback? onGiveFeedbackPressed;
  final VoidCallback? onClosePressed;

  const VerificationDeclinedPage({
    this.onClosePressed,
    this.onGiveFeedbackPressed,
    this.onHistoryPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          Padding(
            padding: const EdgeInsets.all(16),
            child: StatusIcon(
              icon: Icons.not_interested,
              color: Theme.of(context).primaryColorDark,
            ),
          ),
          Padding(
            padding: const EdgeInsets.all(16),
            child: Text(
              locale.verificationScreenDeclinedTitle,
              style: Theme.of(context).textTheme.headline2,
              textAlign: TextAlign.center,
            ),
          ),
          Padding(
            padding: const EdgeInsets.all(16),
            child: Text(
              locale.verificationScreenDeclinedDescription,
              style: Theme.of(context).textTheme.bodyText1,
              textAlign: TextAlign.center,
            ),
          ),
          TextIconButton(
            onPressed: onHistoryPressed,
            child: Text(AppLocalizations.of(context).verificationScreenShowHistoryCta),
          ),
          const Divider(height: 48),
          const Spacer(),
          TextIconButton(
            onPressed: onGiveFeedbackPressed,
            child: Text(locale.verificationScreenGiveFeedbackCta),
          ),
          const SizedBox(height: 16),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: ElevatedButton(
              onPressed: onClosePressed,
              child: Text(AppLocalizations.of(context).verificationScreenCloseCta),
            ),
          ),
        ],
      ),
    );
  }
}
