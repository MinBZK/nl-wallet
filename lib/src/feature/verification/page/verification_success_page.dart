import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/text_icon_button.dart';
import '../widget/success_icon.dart';

class VerificationSuccessPage extends StatelessWidget {
  final String verifierShortName;
  final VoidCallback? onHistoryPressed;
  final VoidCallback? onClosePressed;

  const VerificationSuccessPage({
    required this.verifierShortName,
    this.onClosePressed,
    this.onHistoryPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          const Padding(
            padding: EdgeInsets.symmetric(vertical: 16),
            child: SuccessIcon(),
          ),
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 16),
            child: Text(
              AppLocalizations.of(context).verificationScreenSuccessTitle(verifierShortName),
              style: Theme.of(context).textTheme.headline2,
              textAlign: TextAlign.center,
            ),
          ),
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 16),
            child: Text(
              AppLocalizations.of(context).verificationScreenHistoryDescription,
              style: Theme.of(context).textTheme.bodyText1,
              textAlign: TextAlign.center,
            ),
          ),
          TextIconButton(
            onPressed: onHistoryPressed,
            child: Text(AppLocalizations.of(context).verificationScreenShowHistoryCta),
          ),
          const Spacer(),
          ElevatedButton(
            onPressed: onClosePressed,
            child: Text(AppLocalizations.of(context).verificationScreenCloseCta),
          ),
        ],
      ),
    );
  }
}
