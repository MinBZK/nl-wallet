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
import '../../common/widget/button/list_button.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/secondary_button.dart';
import '../../common/widget/button/tertiary_button.dart';
import '../../common/widget/card/wallet_card_item.dart';
import '../../common/widget/list/list_item.dart';
import '../../common/widget/spacer/sliver_divider.dart';
import '../../common/widget/spacer/sliver_sized_box.dart';
import '../../common/widget/text/body_text.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/wallet_scrollbar.dart';

const kReviewCardsAcceptButtonKey = Key('reviewCardsAcceptButton');
const kReviewCardsDeclineButtonKey = Key('reviewCardsDeclineButton');

class IssuanceReviewCardsPage extends StatelessWidget {
  /// The new cards to be approved by the user
  final List<WalletCard> offeredCards;

  /// The updated cards to be approved by the user
  final List<WalletCard> renewedCards;

  /// Callback triggered when the user accepts, [acceptedCards] are the cards which the user would like to add
  final Function(List<WalletCard> acceptedCards) onAccept;

  /// Callback triggered when the user declines all offered cards
  final VoidCallback onDecline;

  /// Returns a combined list of all offered and renewed cards.
  List<WalletCard> get allCards => offeredCards + renewedCards;

  /// Returns true if there are any cards offered to the user.
  bool get hasOfferedCards => offeredCards.isNotEmpty;

  /// Returns true if there are any cards available for renewal.
  bool get hasRenewedCards => renewedCards.isNotEmpty;

  const IssuanceReviewCardsPage({
    required this.offeredCards,
    required this.renewedCards,
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
                const SliverSizedBox(height: 24),
                _buildOfferedCardsSection(context),
                _buildRenewedCardsSection(context),
              ],
            ),
          ),
          _buildBottomSection(context),
        ],
      ),
    );
  }

  Widget _buildCardsSliver(BuildContext context, List<WalletCard> cards) {
    if (cards.isEmpty) return const SliverSizedBox();
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
      padding: const EdgeInsets.symmetric(horizontal: 16),
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
      icon: const Icon(Icons.info_outline_rounded),
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
        padding: const EdgeInsets.symmetric(horizontal: 8),
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
    final title = _buildTitle(context);
    final subtitle = context.l10n.issuanceReviewCardsPageSubtitle(offeredCards.length, _getOrganizationName(context));

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        children: [
          TitleText(title),
          const SizedBox(height: 8),
          BodyText(subtitle),
        ],
      ),
    );
  }

  String _getOrganizationName(BuildContext context) {
    if (hasOfferedCards) return offeredCards.first.issuer.displayName.l10nValue(context);
    if (hasRenewedCards) return renewedCards.first.issuer.displayName.l10nValue(context);
    return context.l10n.organizationFallbackName;
  }

  String _buildTitle(BuildContext context) {
    if (hasOfferedCards && hasRenewedCards) {
      // Copy for situation where there are both new and updated cards
      return context.l10n.issuanceReviewCardsPageAddAndRenewTitle(offeredCards.length + renewedCards.length);
    } else if (!hasOfferedCards && hasRenewedCards) {
      // Copy for situation where there are only updated cards
      return context.l10n.issuanceReviewCardsPageRenewOnlyTitle(renewedCards.length);
    }
    // Default (only new cards) copy
    return context.l10n.issuanceReviewCardsPageTitle(offeredCards.length);
  }

  Widget _buildBottomSection(BuildContext context) {
    return Align(
      alignment: Alignment.bottomCenter,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Divider(),
          ConfirmButtons(
            primaryButton: PrimaryButton(
              key: kReviewCardsAcceptButtonKey,
              onPressed: () => onAccept(allCards),
              text: Text(context.l10n.issuanceReviewCardsPageAcceptCta(allCards.length)),
            ),
            secondaryButton: SecondaryButton(
              key: kReviewCardsDeclineButtonKey,
              onPressed: onDecline,
              text: Text(context.l10n.issuanceReviewCardsPageDeclineCta),
              icon: const Icon(Icons.block_flipped),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildOfferedCardsSection(BuildContext context) {
    if (!hasOfferedCards) return const SliverSizedBox(height: 0);
    return SliverMainAxisGroup(
      slivers: [
        const SliverDivider(),
        const SliverSizedBox(height: 24),
        _buildCardsSliver(context, offeredCards),
        const SliverSizedBox(height: 24),
      ],
    );
  }

  Widget _buildRenewedCardsSection(BuildContext context) {
    if (!hasRenewedCards) return const SliverSizedBox(height: 0);
    return SliverMainAxisGroup(
      slivers: [
        SliverToBoxAdapter(
          child: ListItem.vertical(
            label: Text(context.l10n.issuanceReviewCardsPageRenewSectionTitle(renewedCards.length)),
            subtitle: Text(context.l10n.issuanceReviewCardsPageRenewSectionSubtitle(renewedCards.length)),
            icon: const Icon(Icons.credit_card_outlined),
            dividerSide: DividerSide.top,
          ),
        ),
        _buildCardsSliver(context, renewedCards),
        const SliverSizedBox(height: 24),
      ],
    );
  }
}
