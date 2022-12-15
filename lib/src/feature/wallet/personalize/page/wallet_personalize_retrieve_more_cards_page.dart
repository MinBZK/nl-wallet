import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../common/widget/explanation_sheet.dart';
import '../../../common/widget/link_button.dart';
import '../../../common/widget/text_icon_button.dart';

class WalletPersonalizeRetrieveMoreCardsPage extends StatelessWidget {
  final VoidCallback onContinuePressed;
  final VoidCallback onSkipPressed;

  const WalletPersonalizeRetrieveMoreCardsPage({
    required this.onContinuePressed,
    required this.onSkipPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
      child: Column(
        mainAxisSize: MainAxisSize.max,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            locale.walletPersonalizeRetrieveMoreCardsPageTitle,
            style: Theme.of(context).textTheme.headline2,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 8),
          Text(
            locale.walletPersonalizeRetrieveMoreCardsPageDescription,
            style: Theme.of(context).textTheme.bodyText1,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 16),
          LinkButton(
            customPadding: EdgeInsets.zero,
            child: Text(locale.walletPersonalizeRetrieveMoreCardsPageWhatIsRetrievedCta),
            onPressed: () {
              ExplanationSheet.show(
                context,
                title: locale.walletPersonalizeRetrieveMoreCardsPageInfoSheetTitle,
                description: locale.walletPersonalizeRetrieveMoreCardsPageInfoSheetDescription,
                closeButtonText: locale.walletPersonalizeRetrieveMoreCardsPageInfoSheetCloseCta,
              );
            },
          ),
          const SizedBox(height: 32),
          Container(
            width: double.infinity,
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: const Color(0xFFF3F4F7),
              borderRadius: BorderRadius.circular(8),
            ),
            height: 105,
            child: const Text('Placeholder image'),
          ),
          const Spacer(),
          ElevatedButton(
            onPressed: onContinuePressed,
            child: Text(locale.walletPersonalizeRetrieveMoreCardsPageContinueCta),
          ),
          const SizedBox(height: 16),
          Center(
            child: TextIconButton(
              onPressed: onSkipPressed,
              child: Text(locale.walletPersonalizeRetrieveMoreCardsPageSkipCta),
            ),
          ),
        ],
      ),
    );
  }
}
