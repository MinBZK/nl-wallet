import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

const _kScanIllustration = 'assets/non-free/images/scan_passport_illustration_1.png';

class WalletPersonalizeScanIdIntroPage extends StatelessWidget {
  final VoidCallback onStartScanPressed;

  const WalletPersonalizeScanIdIntroPage({required this.onStartScanPressed, Key? key}) : super(key: key);

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
            locale.walletPersonalizeScanIdIntroPageTitle,
            style: Theme.of(context).textTheme.headline2,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 8),
          Text(
            locale.walletPersonalizeScanIdIntroPageDescription,
            style: Theme.of(context).textTheme.bodyText1,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 32),
          Image.asset(
            _kScanIllustration,
            width: double.infinity,
            fit: BoxFit.cover,
          ),
          const Spacer(),
          ElevatedButton(onPressed: onStartScanPressed, child: Text(locale.walletPersonalizeScanIdIntroPageContinueCta))
        ],
      ),
    );
  }
}
