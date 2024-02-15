import 'package:flutter/material.dart';

import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/sliver_wallet_app_bar.dart';

class CardDataIncorrectScreen extends StatelessWidget {
  const CardDataIncorrectScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('cardDataIncorrectScreen'),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: Scrollbar(
                thumbVisibility: true,
                child: CustomScrollView(
                  slivers: [
                    SliverWalletAppBar(title: context.l10n.cardDataIncorrectScreenSubhead),
                    SliverToBoxAdapter(
                      child: Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: Text(
                          context.l10n.cardDataIncorrectScreenDescription,
                          style: context.textTheme.bodyLarge,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
            const BottomBackButton()
          ],
        ),
      ),
    );
  }

  static void show(BuildContext context) {
    Navigator.push(
      context,
      SecuredPageRoute(builder: (c) => const CardDataIncorrectScreen()),
    );
  }
}
