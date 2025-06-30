import 'package:collection/collection.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/model/disclosure/disclose_card_request.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/organization_policy.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../util/mapper/context_mapper.dart';
import '../../check_attributes/check_attributes_screen.dart';
import '../../common/sheet/select_card_sheet.dart';
import '../../common/widget/animation/animated_card_switcher.dart';
import '../../common/widget/button/confirm/confirm_buttons.dart';
import '../../common/widget/button/link_button.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/secondary_button.dart';
import '../../common/widget/card/shared_attributes_card.dart';
import '../../common/widget/list/list_item.dart';
import '../../common/widget/spacer/sliver_divider.dart';
import '../../common/widget/spacer/sliver_sized_box.dart';
import '../../common/widget/text/body_text.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/wallet_scrollbar.dart';
import '../../info/info_screen.dart';
import '../../organization/detail/organization_detail_screen.dart';
import '../../organization/widget/organization_row.dart';
import '../../policy/policy_screen.dart';

class DisclosureConfirmDataAttributesPage extends StatelessWidget {
  /// Callback invoked when the user accepts the data sharing request.
  final VoidCallback onAcceptPressed;

  /// Callback invoked when the user declines the data sharing request.
  final VoidCallback onDeclinePressed;

  /// Called when an alternative [WalletCard] is selected for a [DiscloseCardRequest].
  /// The [cardRequest] parameter contains the updated selection.
  final Function(DiscloseCardRequest cardRequest) onAlternativeCardSelected;

  /// The organization/entity requesting access to wallet data.
  final Organization relyingParty;

  /// Inform the user what the purpose is of this request
  final LocalizedText requestPurpose;

  /// List of wallet cards whose attributes are being requested.
  final List<DiscloseCardRequest> cardRequests;

  /// The policy document defining how the relying party handles disclosed data.
  final Policy policy;

  /// Calculates the total number of attributes across all requested wallet cards.
  int get totalNrOfAttributes => cardRequests.map((it) => it.selection.attributes).flattened.length;

  const DisclosureConfirmDataAttributesPage({
    required this.onAcceptPressed,
    required this.onDeclinePressed,
    required this.onAlternativeCardSelected,
    required this.relyingParty,
    required this.cardRequests,
    required this.policy,
    required this.requestPurpose,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: WalletScrollbar(
        child: CustomScrollView(
          restorationId: 'confirm_data_attributes_scrollview',
          slivers: <Widget>[
            const SliverSizedBox(height: 8),
            SliverToBoxAdapter(child: _buildHeaderSection(context)),
            const SliverDivider(),
            SliverToBoxAdapter(
              child: OrganizationRow(
                organization: relyingParty,
                onPressed: () => OrganizationDetailScreen.showPreloaded(context, relyingParty),
              ),
            ),
            const SliverDivider(),
            SliverToBoxAdapter(child: _buildReasonSection(context)),
            const SliverDivider(),
            SliverToBoxAdapter(child: _buildCardsSectionHeader(context)),
            SliverPadding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              sliver: _buildSharedAttributeCardsSliver(),
            ),
            const SliverSizedBox(height: 24),
            const SliverDivider(),
            SliverToBoxAdapter(child: _buildPrivacySection(context)),
            SliverFillRemaining(
              hasScrollBody: false,
              fillOverscroll: true,
              child: _buildBottomSection(context),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSharedAttributeCardsSliver() {
    return SliverList.separated(
      itemCount: cardRequests.length,
      itemBuilder: (context, i) {
        final cardRequest = cardRequests[i];
        final sharedAttributesCard = SharedAttributesCard(
          key: Key(cardRequest.selection.hashCode.toString()),
          card: cardRequest.selection,
          onPressed: () => CheckAttributesScreen.showWithAlternatives(
            context,
            selection: cardRequest.selection,
            cards: cardRequest.candidates,
            onDataIncorrectPressed: () => InfoScreen.showDetailsIncorrect(context),
            onAlternativeCardSelected: (card) => onAlternativeCardSelected(cardRequest.select(card)),
          ),
          onChangeCardPressed: cardRequest.hasAlternatives ? () => _onChangeCardPressed(context, cardRequest) : null,
        );
        final animationEnabled = !context.isScreenReaderEnabled && cardRequest.hasAlternatives;
        return AnimatedCardSwitcher(
          enableAnimation: animationEnabled,
          child: sharedAttributesCard,
        );
      },
      separatorBuilder: (context, i) => const SizedBox(height: 16),
    );
  }

  Future<void> _onChangeCardPressed(BuildContext context, DiscloseCardRequest request) async {
    final userSelection = await SelectCardSheet.show(context, candidates: request.alternatives);
    if (userSelection != null) onAlternativeCardSelected(request.select(userSelection));
  }

  Widget _buildHeaderSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          TitleText(
            context.l10n.disclosureConfirmDataAttributesShareWithTitle(relyingParty.displayName.l10nValue(context)),
          ),
          const SizedBox(height: 8),
          BodyText(
            context.l10n.disclosureConfirmDataAttributesDisclaimer,
          ),
        ],
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        const Divider(),
        ConfirmButtons(
          primaryButton: PrimaryButton(
            key: const Key('acceptButton'),
            onPressed: onAcceptPressed,
            text: Text.rich(context.l10n.disclosureConfirmDataAttributesPageApproveCta.toTextSpan(context)),
          ),
          secondaryButton: SecondaryButton(
            key: const Key('rejectButton'),
            onPressed: onDeclinePressed,
            icon: const Icon(Icons.block_flipped),
            text: Text.rich(context.l10n.disclosureConfirmDataAttributesPageDenyCta.toTextSpan(context)),
          ),
        ),
      ],
    );
  }

  Widget _buildReasonSection(BuildContext context) {
    return ListItem(
      icon: const Icon(Icons.info_outline_rounded),
      label: Text.rich(context.l10n.disclosureConfirmDataAttributesSubtitlePurpose.toTextSpan(context)),
      subtitle: Text.rich(requestPurpose.l10nSpan(context)),
      style: ListItemStyle.vertical,
    );
  }

  Widget _buildCardsSectionHeader(BuildContext context) {
    return ListItem(
      icon: const Icon(Icons.credit_card_outlined),
      label: Text.rich(
        context.l10n.disclosureConfirmDataAttributesSubtitleData(totalNrOfAttributes).toTextSpan(context),
      ),
      subtitle: Text.rich(
        context.l10n.disclosureConfirmDataAttributesSharedAttributesInfo(totalNrOfAttributes).toTextSpan(context),
      ),
      style: ListItemStyle.vertical,
    );
  }

  Widget _buildPrivacySection(BuildContext context) {
    final subtitle = context.read<ContextMapper<OrganizationPolicy, String>>().map(
          context,
          OrganizationPolicy(organization: relyingParty, policy: policy),
        );
    return ListItem(
      icon: const Icon(Icons.handshake_outlined),
      label: Text.rich(context.l10n.disclosureConfirmDataAttributesSubtitleTerms.toTextSpan(context)),
      subtitle: Text.rich(subtitle.toTextSpan(context)),
      style: ListItemStyle.vertical,
      button: LinkButton(
        text: Text.rich(context.l10n.disclosureConfirmDataAttributesCheckConditionsCta.toTextSpan(context)),
        onPressed: () => PolicyScreen.show(context, relyingParty, policy),
      ),
    );
  }
}
