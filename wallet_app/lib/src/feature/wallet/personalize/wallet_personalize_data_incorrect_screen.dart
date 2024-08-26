import 'package:flutter/material.dart';

import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/tertiary_button.dart';
import '../../common/widget/numbered_list.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../common/widget/sliver_wallet_app_bar.dart';
import '../../common/widget/text/body_text.dart';
import '../../common/widget/wallet_scrollbar.dart';

class WalletPersonalizeDataIncorrectScreen extends StatelessWidget {
  final VoidCallback onDataRejected;

  const WalletPersonalizeDataIncorrectScreen({required this.onDataRejected, super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('personalizeDataIncorrectScreen'),
      body: Column(
        children: [
          Expanded(
            child: WalletScrollbar(
              child: SafeArea(
                child: CustomScrollView(
                  slivers: [
                    SliverWalletAppBar(
                      title: context.l10n.walletPersonalizeDataIncorrectScreenSubhead,
                      scrollController: PrimaryScrollController.maybeOf(context),
                    ),
                    SliverToBoxAdapter(
                      child: _buildContent(context),
                    ),
                    const SliverSizedBox(height: 24),
                  ],
                ),
              ),
            ),
          ),
          const Divider(height: 1),
          _buildBottomSection(context),
        ],
      ),
    );
  }

  Widget _buildContent(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          BodyText(
            context.l10n.walletPersonalizeDataIncorrectScreenDescription,
          ),
          const SizedBox(height: 24),
          BodyText(
            context.l10n.walletPersonalizeDataIncorrectScreenHowToResolveTitle,
            style: context.textTheme.bodyLarge?.copyWith(fontWeight: FontWeight.bold),
          ),
          NumberedList(
            items: context.l10n.walletPersonalizeDataIncorrectScreenHowToResolveBulletPoints.split('\n'),
          ),
          const SizedBox(height: 24),
          BodyText(
            context.l10n.walletPersonalizeDataIncorrectScreenNotYourDataTitle,
            style: context.textTheme.bodyLarge?.copyWith(fontWeight: FontWeight.bold),
          ),
          BodyText(
            context.l10n.walletPersonalizeDataIncorrectScreenNotYourDataDescription,
          ),
        ],
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    final verticalPadding = context.isLandscape ? 8.0 : 24.0;
    return SafeArea(
      top: false,
      bottom: true,
      minimum: EdgeInsets.only(bottom: verticalPadding),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16),
        child: Column(
          children: [
            SizedBox(height: verticalPadding),
            PrimaryButton(
              onPressed: onDataRejected,
              text: Text.rich(context.l10n.walletPersonalizeDataIncorrectScreenPrimaryCta.toTextSpan(context)),
            ),
            const SizedBox(height: 12),
            TertiaryButton(
              onPressed: () => Navigator.maybePop(context),
              text: Text.rich(context.l10n.generalBottomBackCta.toTextSpan(context)),
              icon: const Icon(Icons.arrow_back),
            ),
          ],
        ),
      ),
    );
  }

  /// Shows the [WalletPersonalizeDataIncorrectScreen] and also makes sure
  /// the route is popped when the reject button is pressed.
  static void show(BuildContext context, VoidCallback onDataRejected) {
    Navigator.push(
      context,
      SecuredPageRoute(
        builder: (c) => WalletPersonalizeDataIncorrectScreen(
          onDataRejected: () {
            Navigator.pop(c);
            onDataRejected();
          },
        ),
      ),
    );
  }
}
