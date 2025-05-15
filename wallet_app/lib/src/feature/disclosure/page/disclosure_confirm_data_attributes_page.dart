import 'package:collection/collection.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/organization_policy.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/requested_attributes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../util/mapper/context_mapper.dart';
import '../../check_attributes/check_attributes_screen.dart';
import '../../common/widget/button/confirm/confirm_buttons.dart';
import '../../common/widget/button/link_button.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/secondary_button.dart';
import '../../common/widget/card/shared_attributes_card.dart';
import '../../common/widget/spacer/sliver_divider.dart';
import '../../common/widget/spacer/sliver_sized_box.dart';
import '../../common/widget/text/body_text.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/wallet_scrollbar.dart';
import '../../info/info_screen.dart';
import '../../policy/policy_screen.dart';

class DisclosureConfirmDataAttributesPage extends StatelessWidget {
  final VoidCallback onDeclinePressed;
  final VoidCallback onAcceptPressed;

  final Organization relyingParty;
  final RequestedAttributes requestedAttributes;
  final Policy policy;

  /// Inform the user what the purpose is of this request
  final LocalizedText requestPurpose;

  int get totalNrOfAttributes => requestedAttributes.values.map((attributes) => attributes.length).sum;

  const DisclosureConfirmDataAttributesPage({
    required this.onDeclinePressed,
    required this.onAcceptPressed,
    required this.relyingParty,
    required this.requestedAttributes,
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
            SliverToBoxAdapter(child: _buildReasonSection(context)),
            const SliverDivider(),
            const SliverSizedBox(height: 32),
            SliverToBoxAdapter(child: _buildCardsSectionHeader(context)),
            SliverPadding(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
              sliver: _buildSharedAttributeCardsSliver(),
            ),
            const SliverSizedBox(height: 8),
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
      itemCount: requestedAttributes.length,
      itemBuilder: (context, i) {
        final entry = requestedAttributes.entries.elementAt(i);
        return SharedAttributesCard(
          card: entry.key,
          attributes: entry.value,
          onTap: () => CheckAttributesScreen.show(
            context,
            card: entry.key,
            attributes: entry.value,
            onDataIncorrectPressed: () => InfoScreen.showDetailsIncorrect(context),
          ),
        );
      },
      separatorBuilder: (context, i) => const SizedBox(height: 16),
    );
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
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 32),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Icon(Icons.info_outline_rounded, size: 24),
          const SizedBox(height: 16),
          TitleText(
            context.l10n.disclosureConfirmDataAttributesSubtitlePurpose,
            style: context.textTheme.titleLarge,
          ),
          const SizedBox(height: 4),
          BodyText(
            requestPurpose.l10nValue(context),
          ),
        ],
      ),
    );
  }

  Widget _buildCardsSectionHeader(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Icon(Icons.credit_card_outlined, size: 24),
          const SizedBox(height: 16),
          TitleText(
            context.l10n.disclosureConfirmDataAttributesSubtitleData(totalNrOfAttributes),
            style: context.textTheme.titleLarge,
          ),
          const SizedBox(height: 4),
          BodyText(
            context.l10n.disclosureConfirmDataAttributesSharedAttributesInfo(totalNrOfAttributes),
          ),
        ],
      ),
    );
  }

  Widget _buildPrivacySection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 32),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Icon(Icons.handshake_outlined, size: 24),
          const SizedBox(height: 16),
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              TitleText(
                context.l10n.disclosureConfirmDataAttributesSubtitleTerms,
                style: context.textTheme.titleLarge,
              ),
              const SizedBox(height: 4),
              BodyText(
                context.read<ContextMapper<OrganizationPolicy, String>>().map(
                      context,
                      OrganizationPolicy(organization: relyingParty, policy: policy),
                    ),
              ),
            ],
          ),
          const SizedBox(height: 4),
          LinkButton(
            text: Text.rich(context.l10n.disclosureConfirmDataAttributesCheckConditionsCta.toTextSpan(context)),
            onPressed: () => PolicyScreen.show(context, relyingParty, policy),
          ),
        ],
      ),
    );
  }
}
