import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/placeholder_screen.dart';

class WalletPersonalizeDataIncorrectScreen extends StatelessWidget {
  const WalletPersonalizeDataIncorrectScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(context.l10n.walletPersonalizeDataIncorrectScreenTitle),
      ),
      body: CustomScrollView(
        slivers: [
          SliverToBoxAdapter(
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 32),
              child: MergeSemantics(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      context.l10n.walletPersonalizeDataIncorrectScreenSubhead,
                      style: context.textTheme.headlineMedium,
                    ),
                    const SizedBox(height: 8),
                    Text(
                      context.l10n.walletPersonalizeDataIncorrectScreenDescription,
                      style: context.textTheme.bodyLarge,
                    ),
                  ],
                ),
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
              child: Text(context.l10n.walletPersonalizeDataIncorrectScreenPrimaryCta),
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
