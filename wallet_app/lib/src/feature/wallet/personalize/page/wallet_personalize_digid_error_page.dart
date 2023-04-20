import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../common/widget/button/text_icon_button.dart';

const _kDigidLogoPath = 'assets/images/digid_logo.png';

class WalletPersonalizeDigidErrorPage extends StatelessWidget {
  final VoidCallback onRetryPressed;
  final VoidCallback onHelpPressed;

  const WalletPersonalizeDigidErrorPage({
    required this.onRetryPressed,
    required this.onHelpPressed,
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
          const SizedBox(height: 12),
          SizedBox(
            height: 105,
            child: Container(
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.secondaryContainer,
                borderRadius: BorderRadius.circular(8),
              ),
              alignment: Alignment.center,
              child: const Text('Placeholder image'),
            ),
          ),
          const SizedBox(height: 24),
          Text(
            locale.walletPersonalizeDigidErrorPageTitle,
            textAlign: TextAlign.start,
            style: Theme.of(context).textTheme.displaySmall,
          ),
          const SizedBox(height: 8),
          Text(
            locale.walletPersonalizeDigidErrorPageDescription,
            textAlign: TextAlign.start,
            style: Theme.of(context).textTheme.bodyLarge,
          ),
          const Spacer(),
          ElevatedButton(
            onPressed: onRetryPressed,
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Image.asset(_kDigidLogoPath),
                const SizedBox(width: 12),
                Text(locale.walletPersonalizeDigidErrorPageLoginWithDigidCta),
              ],
            ),
          ),
          const SizedBox(height: 8),
          Center(
            child: TextIconButton(
              onPressed: onHelpPressed,
              child: Text(
                locale.walletPersonalizeDigidErrorPageNoDigidCta,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
