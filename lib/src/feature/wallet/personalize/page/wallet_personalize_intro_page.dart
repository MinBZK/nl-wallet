import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../common/widget/text_icon_button.dart';

const _kIllustrationPath = 'assets/images/personalize_wallet_intro_illustration.png';
const _kDigidLogoPath = 'assets/images/digid_logo.png';

class WalletPersonalizeIntroPage extends StatelessWidget {
  final VoidCallback onLoginWithDigidPressed;
  final VoidCallback onNoDigidPressed;

  const WalletPersonalizeIntroPage({
    required this.onLoginWithDigidPressed,
    required this.onNoDigidPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
      child: Column(
        mainAxisSize: MainAxisSize.max,
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          const SizedBox(height: 12),
          Text(
            locale.walletPersonalizeIntroPageTitle,
            textAlign: TextAlign.center,
            style: Theme.of(context).textTheme.headline3,
          ),
          const SizedBox(height: 8),
          Text(
            locale.walletPersonalizeIntroPageDescription,
            textAlign: TextAlign.center,
            style: Theme.of(context).textTheme.bodyText1,
          ),
          const Spacer(),
          Image.asset(_kIllustrationPath),
          const Spacer(flex: 3),
          ElevatedButton(
            onPressed: onLoginWithDigidPressed,
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Image.asset(_kDigidLogoPath),
                const SizedBox(width: 12),
                Text(locale.walletPersonalizeIntroPageLoginWithDigidCta),
              ],
            ),
          ),
          const SizedBox(height: 8),
          TextIconButton(
            onPressed: onNoDigidPressed,
            child: Text(locale.walletPersonalizeIntroPageNoDigidCta),
          ),
        ],
      ),
    );
  }
}
