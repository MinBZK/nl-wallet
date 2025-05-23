import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter_staggered_grid_view/flutter_staggered_grid_view.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/card/wallet_card.dart';
import '../../../theme/wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_constants.dart';
import '../../card/preview/card_preview_screen.dart';
import '../../common/widget/button/confirm/confirm_buttons.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/secondary_button.dart';
import '../../common/widget/button/tertiary_button.dart';
import '../../common/widget/card/wallet_card_item.dart';
import '../../common/widget/spacer/sliver_divider.dart';
import '../../common/widget/spacer/sliver_sized_box.dart';
import '../../common/widget/text/body_text.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/wallet_scrollbar.dart';

const kReviewCardsAcceptButtonKey = Key('reviewCardsAcceptButton');
const kReviewCardsDeclineButtonKey = Key('reviewCardsDeclineButton');

class IssuanceReviewCardsPage extends StatelessWidget {
  /// The cards to be approved by the user
  final List<WalletCard> cards;

  /// Callback triggered when the user accepts, [acceptedCards] are the cards which the user would like to add
  final Function(List<WalletCard> acceptedCards) onAccept;

  /// Callback triggered when the user declines all offered cards
  final VoidCallback onDecline;

  const IssuanceReviewCardsPage({
    required this.cards,
    required this.onAccept,
    required this.onDecline,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return WalletScrollbar(
      child: Column(
        children: [
          Expanded(
            child: CustomScrollView(
              restorationId: 'issuance_review_cards_page',
              slivers: <Widget>[
                const SliverSizedBox(height: 24),
                SliverToBoxAdapter(child: _buildHeaderSection(context)),
                const SliverDivider(height: 48),
                _buildCardsSliver(context),
                const SliverSizedBox(height: 24),
              ],
            ),
          ),
          _buildBottomSection(context),
        ],
      ),
    );
  }

  Widget _buildCardsSliver(BuildContext context) {
    final crossAxisCount = max(1, (context.mediaQuery.size.width / kCardBreakPointWidth).floor());
    return SliverMasonryGrid(
      gridDelegate: SliverSimpleGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: min(crossAxisCount, 2 /* max columns */),
      ),
      mainAxisSpacing: 16,
      crossAxisSpacing: 16,
      delegate: SliverChildBuilderDelegate(
        (context, index) => _buildCardListItem(context, cards[index]),
        childCount: cards.length,
      ),
    );
  }

  Widget _buildCardListItem(BuildContext context, WalletCard card) {
    return Padding(
      padding: EdgeInsets.symmetric(horizontal: 16),
      child: Stack(
        fit: StackFit.passthrough,
        children: [
          WalletCardItem.fromWalletCard(context, card),
          Positioned(bottom: 0, left: 0, right: 0, child: _buildCardButtons(context, card: card)),
        ],
      ),
    );
  }

  Widget _buildCardButtons(BuildContext context, {bool showAddButton = false, required WalletCard card}) {
    final checkDetailsButton = TertiaryButton(
      onPressed: () => CardPreviewScreen.show(context, card: card),
      mainAxisAlignment: MainAxisAlignment.start,
      text: Text(context.l10n.issuanceReviewCardsPageShowDetailsCta),
      icon: Icon(Icons.info_outline_rounded),
    );

    final addButton = Align(
      alignment: Alignment.centerLeft,
      child: IntrinsicWidth(
        child: TertiaryButton(
          mainAxisAlignment: MainAxisAlignment.start,
          onPressed: () {},
          text: Text(context.l10n.issuanceReviewCardsPageToggleAddCta),
          icon: Checkbox(
            value: true,
            onChanged: null /* avoid it grabbing individual focus */,
            fillColor: WidgetStatePropertyAll(context.colorScheme.primary) /* override disabled color */,
          ),
        ),
      ),
    );

    return DecoratedBox(
      decoration: BoxDecoration(
        color: context.colorScheme.surface,
        borderRadius: BorderRadius.vertical(
          bottom: WalletTheme.kBorderRadius12.bottomLeft,
        ),
      ),
      child: Padding(
        padding: EdgeInsets.symmetric(horizontal: 8),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            if (showAddButton) Expanded(child: addButton),
            Expanded(child: checkDetailsButton),
          ],
        ),
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    final title = context.l10n.issuanceReviewCardsPageTitle(cards.length);
    final subtitle = context.l10n.issuanceReviewCardsPageSubtitle(
      cards.length,
      cards.first.issuer.displayName.l10nValue(context),
    );

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        children: [
          TitleText(title),
          BodyText(subtitle),
        ],
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Align(
      alignment: Alignment.bottomCenter,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Divider(),
          ConfirmButtons(
            primaryButton: PrimaryButton(
              key: kReviewCardsAcceptButtonKey,
              onPressed: () => onAccept(cards),
              text: Text(context.l10n.issuanceReviewCardsPageAcceptCta(cards.length)),
            ),
            secondaryButton: SecondaryButton(
              key: kReviewCardsDeclineButtonKey,
              onPressed: onDecline,
              text: Text(context.l10n.issuanceReviewCardsPageDeclineCta),
              icon: Icon(Icons.block_flipped),
            ),
          ),
        ],
      ),
    );
  }
}
