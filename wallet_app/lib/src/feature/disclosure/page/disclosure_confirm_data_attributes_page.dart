import 'package:collection/collection.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import '../../../data/repository/organization/organization_repository.dart';
import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/duration_extension.dart';
import '../../check_attributes/check_attributes_screen.dart';
import '../../common/widget/button/confirm_buttons.dart';
import '../../common/widget/button/link_button.dart';
import '../../common/widget/card/shared_attributes_card.dart';
import '../../common/widget/sliver_divider.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../policy/policy_screen.dart';

class DisclosureConfirmDataAttributesPage extends StatelessWidget {
  final VoidCallback onDeclinePressed;
  final VoidCallback onAcceptPressed;
  final VoidCallback? onReportIssuePressed;

  final Organization relyingParty;
  final Map<WalletCard, List<DataAttribute>> requestedAttributes;
  final Policy policy;

  /// Inform the user what the purpose is of this request
  final LocalizedText requestPurpose;

  int get totalNrOfAttributes => requestedAttributes.values.map((attributes) => attributes.length).sum;

  const DisclosureConfirmDataAttributesPage({
    required this.onDeclinePressed,
    required this.onAcceptPressed,
    this.onReportIssuePressed,
    required this.relyingParty,
    required this.requestedAttributes,
    required this.policy,
    required this.requestPurpose,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      thumbVisibility: true,
      trackVisibility: true,
      child: CustomScrollView(
        restorationId: 'confirm_data_attributes_scrollview',
        slivers: <Widget>[
          const SliverSizedBox(height: 8),
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          const SliverDivider(height: 1),
          SliverToBoxAdapter(child: _buildReasonSection(context)),
          const SliverDivider(height: 1),
          const SliverSizedBox(height: 32),
          SliverToBoxAdapter(child: _buildCardsSectionHeader(context)),
          SliverPadding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
            sliver: _buildSharedAttributeCardsSliver(),
          ),
          const SliverSizedBox(height: 8),
          const SliverDivider(height: 1),
          SliverToBoxAdapter(child: _buildPrivacySection(context)),
          SliverFillRemaining(
            hasScrollBody: false,
            fillOverscroll: true,
            child: _buildBottomSection(context),
          ),
        ],
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
            onDataIncorrectPressed: () {
              Navigator.pop(context);
              onReportIssuePressed?.call();
            },
          ),
        );
      },
      separatorBuilder: (context, i) => const SizedBox(height: 16),
    );
  }

  String _buildConditionsText(BuildContext context) {
    bool dataIsStored = policy.storageDuration != null;
    if (policy.dataIsShared && !dataIsStored) {
      // Data IS shared but NOT stored
      return context.l10n.disclosureConfirmDataAttributesPageSharedNotStoredSubtitle;
    } else if (policy.dataIsShared && dataIsStored) {
      // Data IS shared and IS stored
      return context.l10n.disclosureConfirmDataAttributesPageSharedAndStoredSubtitle(policy.storageDuration!.inMonths);
    } else if (!policy.dataIsShared && !dataIsStored) {
      // Data is NOT shared and NOT stored
      return context.l10n.disclosureConfirmDataAttributesPageNotSharedNotStoredSubtitle;
    } else if (!policy.dataIsShared && dataIsStored) {
      // Data is NOT shared but IS stored
      return context.l10n
          .disclosureConfirmDataAttributesPageNotSharedButStoredSubtitle(policy.storageDuration!.inMonths);
    }
    if (kDebugMode) throw UnsupportedError('No valid condition combination found');
    return '';
  }

  Widget _buildHeaderSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: MergeSemantics(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              context.l10n.disclosureConfirmDataAttributesShareWithTitle(relyingParty.displayName.l10nValue(context)),
              style: context.textTheme.displayMedium,
              textAlign: TextAlign.start,
            ),
            const SizedBox(height: 8),
            Text(
              context.l10n.disclosureConfirmDataAttributesDisclaimer,
              style: context.textTheme.bodyLarge,
              textAlign: TextAlign.start,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        const Divider(height: 1),
        ConfirmButtons(
          onAcceptPressed: onAcceptPressed,
          acceptText: context.l10n.disclosureConfirmDataAttributesPageApproveCta,
          onDeclinePressed: onDeclinePressed,
          acceptIcon: Icons.arrow_forward,
          declineText: context.l10n.disclosureConfirmDataAttributesPageDenyCta,
        ),
      ],
    );
  }

  Widget _buildReasonSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 32),
      child: MergeSemantics(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Icon(Icons.info_outline_rounded, size: 24),
            const SizedBox(height: 16),
            Text(
              context.l10n.disclosureConfirmDataAttributesSubtitlePurpose,
              style: context.textTheme.displaySmall,
              textAlign: TextAlign.start,
            ),
            const SizedBox(height: 4),
            Text(
              requestPurpose.l10nValue(context),
              style: context.textTheme.bodyLarge,
              textAlign: TextAlign.start,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildCardsSectionHeader(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: MergeSemantics(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Icon(Icons.credit_card_outlined, size: 24),
            const SizedBox(height: 16),
            Text(
              context.l10n.disclosureConfirmDataAttributesSubtitleData,
              style: context.textTheme.displaySmall,
              textAlign: TextAlign.start,
            ),
            const SizedBox(height: 4),
            Text(
              context.l10n.disclosureConfirmDataAttributesSharedAttributesInfo(totalNrOfAttributes),
              style: context.textTheme.bodyLarge,
              textAlign: TextAlign.start,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildPrivacySection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 32),
      child: MergeSemantics(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Icon(Icons.handshake_outlined, size: 24),
            const SizedBox(height: 16),
            Text(
              context.l10n.disclosureConfirmDataAttributesSubtitleTerms,
              style: context.textTheme.displaySmall,
              textAlign: TextAlign.start,
            ),
            const SizedBox(height: 4),
            Text(
              _buildConditionsText(context),
              style: context.textTheme.bodyLarge,
              textAlign: TextAlign.start,
            ),
            const SizedBox(height: 4),
            LinkButton(
              customPadding: EdgeInsets.zero,
              child: Text(context.l10n.disclosureConfirmDataAttributesCheckConditionsCta),
              onPressed: () => PolicyScreen.show(
                context,
                policy,
                onReportIssuePressed: onReportIssuePressed,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
