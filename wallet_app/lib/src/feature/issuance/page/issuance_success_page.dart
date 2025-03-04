import 'package:flutter/material.dart';

import '../../../domain/model/card/wallet_card.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../common/widget/stacked_wallet_cards.dart';
import '../../common/widget/status_icon.dart';
import '../../common/widget/wallet_scrollbar.dart';

class IssuanceSuccessPage extends StatelessWidget {
  final VoidCallback onClose;
  final List<WalletCard> cards;
  final bool isRefreshFlow;

  const IssuanceSuccessPage({
    required this.onClose,
    required this.cards,
    required this.isRefreshFlow,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return WalletScrollbar(
      child: CustomScrollView(
        restorationId: 'issuance_success_page',
        slivers: <Widget>[
          const SliverSizedBox(height: 48),
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          const SliverSizedBox(height: 32),
          SliverToBoxAdapter(
            child: Column(
              children: [
                const SizedBox(height: 16),
                Padding(
                  padding: const EdgeInsets.all(16),
                  child: StackedWalletCards(cards: cards),
                ),
              ],
            ),
          ),
          const SliverSizedBox(height: 16),
          SliverFillRemaining(hasScrollBody: false, fillOverscroll: true, child: _buildBottomSection(context)),
        ],
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    final subtitle = isRefreshFlow
        ? context.l10n.issuanceSuccessPageCardsUpdatedSubtitle(cards.length)
        : context.l10n.issuanceSuccessPageCardsAddedSubtitle(cards.length);

    return MergeSemantics(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: StatusIcon(
              icon: Icons.check,
              color: context.colorScheme.primary,
            ),
          ),
          const SizedBox(height: 32),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Text(
              context.l10n.issuanceSuccessPageTitle,
              style: context.textTheme.displayMedium,
              textAlign: TextAlign.center,
            ),
          ),
          const SizedBox(height: 8),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Text(
              subtitle,
              style: context.textTheme.bodyLarge,
              textAlign: TextAlign.center,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Align(
      alignment: Alignment.bottomCenter,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: SizedBox(
          height: 48,
          child: ElevatedButton(
            onPressed: onClose,
            child: Text.rich(context.l10n.issuanceSuccessPageCloseCta.toTextSpan(context)),
          ),
        ),
      ),
    );
  }
}
