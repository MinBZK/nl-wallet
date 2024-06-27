import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/event/wallet_event.dart';
import '../../../../domain/model/organization.dart';
import '../../../../domain/model/policy/policy.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/wallet_event_extension.dart';
import '../../../../util/mapper/context_mapper.dart';
import '../../../check_attributes/check_attributes_screen.dart';
import '../../../common/screen/placeholder_screen.dart';
import '../../../common/widget/button/list_button.dart';
import '../../../common/widget/card/shared_attributes_card.dart';
import '../../../common/widget/sliver_divider.dart';
import '../../../common/widget/sliver_sized_box.dart';
import '../../../common/widget/text/body_text.dart';
import '../../../info/info_screen.dart';
import '../../../organization/detail/organization_detail_screen.dart';
import '../../../policy/policy_screen.dart';
import 'wallet_event_status_header.dart';

class HistoryDetailCommonBuilders {
  HistoryDetailCommonBuilders._();

  static Widget buildStatusHeaderSliver(BuildContext context, WalletEvent event) {
    return SliverMainAxisGroup(
      slivers: [
        SliverToBoxAdapter(
          child: WalletEventStatusHeader(event: event),
        ),
        const SliverDivider(),
      ],
    );
  }

  static Widget buildPurposeSliver(BuildContext context, DisclosureEvent event) {
    return SliverMainAxisGroup(
      slivers: [
        SliverToBoxAdapter(
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Icon(
                  Icons.info_outline_rounded,
                  size: 24,
                  color: context.colorScheme.onSurfaceVariant,
                ),
                const SizedBox(height: 16),
                BodyText(
                  context.l10n.historyDetailScreenPurposeTitle,
                  style: context.textTheme.headlineMedium,
                ),
                const SizedBox(height: 8),
                BodyText(event.purpose.l10nValue(context)),
              ],
            ),
          ),
        ),
        const SliverDivider(),
      ],
    );
  }

  static Widget buildAttributesSliver(BuildContext context, DisclosureEvent event) {
    final attributesByDocType = event.attributesByDocType;
    final totalNrOfAttributes = attributesByDocType.values.fold(0, (prev, element) => prev + element.length);
    final attributesSliver = SliverList.separated(
      itemCount: attributesByDocType.length,
      itemBuilder: (context, i) {
        final entry = attributesByDocType.entries.elementAt(i);
        final card = event.cards.firstWhere((card) => card.docType == entry.key);
        return SharedAttributesCard(
          card: card,
          attributes: entry.value,
          onTap: () => CheckAttributesScreen.show(
            context,
            card: card,
            attributes: entry.value,
            onDataIncorrectPressed: () => InfoScreen.showDetailsIncorrect(context),
          ),
        );
      },
      separatorBuilder: (context, i) => const SizedBox(height: 16),
    );
    return SliverMainAxisGroup(
      slivers: [
        SliverToBoxAdapter(
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Icon(
                  Icons.credit_card_outlined,
                  size: 24,
                  color: context.colorScheme.onSurfaceVariant,
                ),
                const SizedBox(height: 16),
                BodyText(
                  context.l10n.historyDetailScreenSharedAttributesTitle,
                  style: context.textTheme.headlineMedium,
                ),
                const SizedBox(height: 8),
                BodyText(
                  context.l10n.historyDetailScreenSharedAttributesSubtitle(totalNrOfAttributes),
                ),
              ],
            ),
          ),
        ),
        SliverPadding(
          sliver: attributesSliver,
          padding: const EdgeInsets.symmetric(horizontal: 16),
        ),
        const SliverSizedBox(height: 24),
        const SliverDivider(),
      ],
    );
  }

  static Widget buildPolicySliver(BuildContext context, Policy policy) {
    final policyTextMapper = context.read<ContextMapper<Policy, String>>();
    return SliverMainAxisGroup(
      slivers: [
        const SliverSizedBox(height: 24),
        SliverToBoxAdapter(
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Icon(
                  Icons.handshake_outlined,
                  size: 24,
                  color: context.colorScheme.onSurfaceVariant,
                ),
                const SizedBox(height: 16),
                BodyText(
                  context.l10n.historyDetailScreenTermsTitle,
                  style: context.textTheme.headlineMedium,
                ),
                const SizedBox(height: 8),
                BodyText(
                  policyTextMapper.map(context, policy),
                ),
              ],
            ),
          ),
        ),
        SliverToBoxAdapter(
          child: ListButton(
            text: Text(context.l10n.historyDetailScreenTermsCta),
            dividerSide: DividerSide.none,
            onPressed: () => PolicyScreen.show(context, policy),
          ),
        ),
        const SliverDivider(),
      ],
    );
  }

  static Widget buildAboutOrganizationSliver(BuildContext context, Organization organization) {
    return SliverToBoxAdapter(
      child: ListButton(
        text: Text(context.l10n.historyDetailScreenAboutOrganizationCta(organization.displayName.l10nValue(context))),
        onPressed: () => OrganizationDetailScreen.showPreloaded(
          context,
          organization,
          sharedDataWithOrganizationBefore: false,
          onReportIssuePressed: () => PlaceholderScreen.showGeneric(context),
        ),
        dividerSide: DividerSide.bottom,
      ),
    );
  }

  static Widget buildShowDetailsSliver(BuildContext context, WalletEvent event) {
    return SliverToBoxAdapter(
      child: ListButton(
        text: Text(context.l10n.historyDetailScreenShowDetailsCta),
        onPressed: () => PlaceholderScreen.showGeneric(context),
        dividerSide: DividerSide.bottom,
      ),
    );
  }

  static Widget buildReportIssueSliver(BuildContext context) {
    return SliverToBoxAdapter(
      child: ListButton(
        text: Text(context.l10n.historyDetailScreenReportIssueCta),
        onPressed: () => PlaceholderScreen.showGeneric(context),
        dividerSide: DividerSide.bottom,
      ),
    );
  }
}
