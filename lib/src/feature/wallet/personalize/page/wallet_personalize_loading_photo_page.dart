import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

class WalletPersonalizeLoadingPhotoPage extends StatelessWidget {
  final Duration mockDelay;

  const WalletPersonalizeLoadingPhotoPage({required this.mockDelay, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          Text(
            locale.walletPersonalizeLoadingPhotoPageTitle,
            style: Theme.of(context).textTheme.headline4,
          ),
          const SizedBox(height: 24),
          _buildProgressIndicator(),
          const SizedBox(height: 24),
          Text(
            locale.walletPersonalizeLoadingPhotoPageSubtitle,
            style: Theme.of(context).textTheme.bodyText1,
          ),
        ],
      ),
    );
  }

  Widget _buildProgressIndicator() {
    return TweenAnimationBuilder<double>(
      builder: (context, progress, child) => CircularProgressIndicator(value: progress),
      duration: mockDelay,
      tween: Tween<double>(begin: 0, end: 1),
    );
  }
}
