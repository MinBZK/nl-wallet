import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../common/widget/placeholder_screen.dart';
import '../../../common/widget/text_icon_button.dart';

const _kDigidLogoPath = 'assets/images/digid_logo.png';

class WalletPersonalizeNoDigidPage extends StatelessWidget {
  const WalletPersonalizeNoDigidPage({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Scaffold(
      appBar: AppBar(
        title: Text(locale.walletPersonalizeNoDigidPageTitle),
      ),
      body: Padding(
        padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
        child: Column(
          mainAxisSize: MainAxisSize.max,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const SizedBox(height: 12),
            Text(
              locale.walletPersonalizeNoDigidPageHeadline,
              textAlign: TextAlign.start,
              style: Theme.of(context).textTheme.displayMedium,
            ),
            const SizedBox(height: 8),
            Text(
              locale.walletPersonalizeNoDigidPageDescription,
              textAlign: TextAlign.start,
              style: Theme.of(context).textTheme.bodyLarge,
            ),
            const Spacer(),
            ElevatedButton(
              onPressed: () => PlaceholderScreen.show(context),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Image.asset(_kDigidLogoPath),
                  const SizedBox(width: 12),
                  Text(locale.walletPersonalizeNoDigidPageRequestDigidCta),
                ],
              ),
            ),
            const SizedBox(height: 8),
            Center(
              child: TextIconButton(
                icon: Icons.arrow_back,
                iconPosition: IconPosition.start,
                onPressed: () => Navigator.pop(context),
                child: Text(locale.walletPersonalizeNoDigidPageBackCta),
              ),
            ),
          ],
        ),
      ),
    );
  }

  static void show(BuildContext context) {
    Navigator.push(
      context,
      MaterialPageRoute(builder: (c) => const WalletPersonalizeNoDigidPage()),
    );
  }
}
