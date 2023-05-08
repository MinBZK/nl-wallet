import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/placeholder_screen.dart';

class WalletPersonalizeDataIncorrectScreen extends StatelessWidget {
  const WalletPersonalizeDataIncorrectScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Scaffold(
      appBar: AppBar(
        title: Text(locale.walletPersonalizeDataIncorrectScreenTitle),
      ),
      body: CustomScrollView(
        slivers: [
          SliverToBoxAdapter(
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 32),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    locale.walletPersonalizeDataIncorrectScreenSubhead,
                    style: Theme.of(context).textTheme.headlineMedium,
                  ),
                  const SizedBox(height: 8),
                  Text(
                    locale.walletPersonalizeDataIncorrectScreenDescription,
                    style: Theme.of(context).textTheme.bodyLarge,
                  ),
                ],
              ),
            ),
          ),
          SliverFillRemaining(
            hasScrollBody: false,
            fillOverscroll: true,
            child: _buildBottomSection(context),
          ),
        ],
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) => Container(
      alignment: Alignment.bottomCenter,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: ElevatedButton(
              onPressed: () => PlaceholderScreen.show(context),
              child: Text(AppLocalizations.of(context).walletPersonalizeDataIncorrectScreenPrimaryCta),
             ),
          ),
          const BottomBackButton(),
        ],
      ));

  static void show(BuildContext context) {
    Navigator.push(
      context,
      MaterialPageRoute(builder: (c) => const WalletPersonalizeDataIncorrectScreen()),
    );
  }
}
