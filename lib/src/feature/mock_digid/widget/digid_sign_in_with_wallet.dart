import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/wallet_logo.dart';

class DigidSignInWithWallet extends StatelessWidget {
  const DigidSignInWithWallet({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        const WalletLogo(size: 80),
        const SizedBox(height: 8),
        Text(AppLocalizations.of(context).appTitle,
            style: Theme.of(context).textTheme.headline2?.copyWith(fontWeight: FontWeight.bold)),
      ],
    );
  }
}
