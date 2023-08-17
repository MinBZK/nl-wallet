import 'package:flutter/material.dart';

import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/button/bottom_back_button.dart';

class WalletPersonalizeDataIncorrectScreen extends StatelessWidget {
  const WalletPersonalizeDataIncorrectScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(context.l10n.walletPersonalizeDataIncorrectScreenTitle),
      ),
      body: SafeArea(
        child: CustomScrollView(
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
                        style: context.textTheme.displayMedium,
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
            const SliverFillRemaining(
              hasScrollBody: false,
              fillOverscroll: true,
              child: BottomBackButton(),
            ),
          ],
        ),
      ),
    );
  }

  static void show(BuildContext context) {
    Navigator.push(
      context,
      SecuredPageRoute(builder: (c) => const WalletPersonalizeDataIncorrectScreen()),
    );
  }
}
