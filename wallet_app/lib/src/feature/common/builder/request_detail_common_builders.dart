import 'package:collection/collection.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/model/event/wallet_event.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/organization_policy.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../util/mapper/context_mapper.dart';
import '../../check_attributes/check_attributes_screen.dart';
import '../../history/detail/widget/wallet_event_status_header.dart';
import '../../info/info_screen.dart';
import '../../organization/detail/organization_detail_screen.dart';
import '../../policy/policy_screen.dart';
import '../screen/placeholder_screen.dart';
import '../screen/request_details_screen.dart';
import '../widget/app_image.dart';
import '../widget/button/link_button.dart';
import '../widget/button/list_button.dart';
import '../widget/card/shared_attributes_card.dart';
import '../widget/list/list_item.dart';
import '../widget/spacer/sliver_divider.dart';
import '../widget/spacer/sliver_sized_box.dart';

class RequestDetailCommonBuilders {
  RequestDetailCommonBuilders._();

  static Widget buildStatusHeaderSliver(
    BuildContext context, {
    required WalletEvent event,
    DividerSide side = DividerSide.none,
  }) {
    return SliverMainAxisGroup(
      slivers: [
        if (side.top) const SliverDivider(),
        SliverToBoxAdapter(
          child: WalletEventStatusHeader(event: event),
        ),
        if (side.bottom) const SliverDivider(),
      ],
    );
  }

  static Widget buildPurposeSliver(
    BuildContext context, {
    required LocalizedText purpose,
    DividerSide side = DividerSide.none,
  }) {
    return SliverToBoxAdapter(
      child: ListItem(
        label: Text.rich(context.l10n.historyDetailScreenPurposeTitle.toTextSpan(context)),
        subtitle: Text.rich(purpose.l10nSpan(context)),
        icon: const Icon(Icons.info_outline_rounded),
        style: ListItemStyle.vertical,
        dividerSide: side,
      ),
    );
  }

  static Widget buildSharedAttributesSliver(
    BuildContext context, {
    required List<WalletCard> cards,
    DividerSide side = DividerSide.none,
  }) {
    final totalNrOfAttributes = cards.map((it) => it.attributes).flattened.length;
    final String title = context.l10n.historyDetailScreenSharedAttributesTitle;
    final subtitle = context.l10n.historyDetailScreenSharedAttributesSubtitle(totalNrOfAttributes);
    return _buildAttributesSliver(context, cards: cards, title: title, subtitle: subtitle, side: side);
  }

  static Widget buildRequestedAttributesSliver(
    BuildContext context, {
    required List<WalletCard> cards,
    DividerSide side = DividerSide.none,
  }) {
    final totalNrOfAttributes = cards.map((it) => it.attributes).flattened.length;
    final String title = context.l10n.requestDetailsScreenAttributesTitle;
    final subtitle = context.l10n.requestDetailsScreenAttributesSubtitle(totalNrOfAttributes);
    return _buildAttributesSliver(context, cards: cards, title: title, subtitle: subtitle, side: side);
  }

  static Widget _buildAttributesSliver(
    BuildContext context, {
    required List<WalletCard> cards,
    required String title,
    required String subtitle,
    DividerSide side = DividerSide.none,
  }) {
    final headerSliver = SliverToBoxAdapter(
      child: ListItem(
        label: Text.rich(title.toTextSpan(context)),
        subtitle: Text.rich(subtitle.toTextSpan(context)),
        icon: const Icon(Icons.credit_card_outlined),
        style: ListItemStyle.vertical,
        dividerSide: DividerSide.none /* handled below */,
      ),
    );

    final attributesSliver = SliverList.separated(
      itemCount: cards.length,
      itemBuilder: (context, i) {
        final card = cards[i];
        return SharedAttributesCard(
          card: card,
          attributes: card.attributes,
          onTap: () => CheckAttributesScreen.show(
            context,
            card: card,
            attributes: card.attributes,
            onDataIncorrectPressed: () => InfoScreen.showDetailsIncorrect(context),
          ),
        );
      },
      separatorBuilder: (context, i) => const SizedBox(height: 16),
    );
    return SliverMainAxisGroup(
      slivers: [
        if (side.top) const SliverDivider(),
        headerSliver,
        SliverPadding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          sliver: attributesSliver,
        ),
        const SliverSizedBox(height: 24),
        if (side.bottom) const SliverDivider(),
      ],
    );
  }

  static Widget buildPolicySliver(
    BuildContext context, {
    required Organization organization,
    required Policy policy,
    DividerSide side = DividerSide.none,
  }) {
    final OrganizationPolicy orgPolicy = OrganizationPolicy(policy: policy, organization: organization);
    final policyTextMapper = context.read<ContextMapper<OrganizationPolicy, String>>();
    return SliverToBoxAdapter(
      child: ListItem(
        label: Text.rich(context.l10n.historyDetailScreenTermsTitle.toTextSpan(context)),
        subtitle: Text.rich(policyTextMapper.map(context, orgPolicy).toTextSpan(context)),
        icon: const Icon(Icons.handshake_outlined),
        button: LinkButton(
          text: Text.rich(context.l10n.historyDetailScreenTermsCta.toTextSpan(context)),
          onPressed: () => PolicyScreen.show(context, organization, policy),
        ),
        style: ListItemStyle.vertical,
        dividerSide: side,
      ),
    );
  }

  static Widget buildAboutOrganizationSliver(
    BuildContext context, {
    required Organization organization,
    DividerSide side = DividerSide.none,
  }) {
    return SliverToBoxAdapter(
      child: ListButton(
        text: Text.rich(
          context.l10n
              .historyDetailScreenAboutOrganizationCta(organization.displayName.l10nValue(context))
              .toTextSpan(context),
        ),
        onPressed: () => OrganizationDetailScreen.showPreloaded(
          context,
          organization,
          sharedDataWithOrganizationBefore: false,
          onReportIssuePressed: () => PlaceholderScreen.showGeneric(context),
        ),
        dividerSide: side,
        trailing: ExcludeSemantics(
          child: SizedBox(
            height: 36,
            width: 36,
            child: AppImage(asset: organization.logo),
          ),
        ),
      ),
    );
  }

  static Widget buildShowDetailsSliver(
    BuildContext context, {
    required DisclosureEvent event,
    DividerSide side = DividerSide.none,
  }) {
    return SliverToBoxAdapter(
      child: ListButton(
        text: Text.rich(context.l10n.historyDetailScreenShowDetailsCta.toTextSpan(context)),
        onPressed: () => RequestDetailsScreen.showEvent(context, event),
        dividerSide: side,
      ),
    );
  }

  static Widget buildReportIssueSliver(BuildContext context, {DividerSide side = DividerSide.none}) {
    return SliverToBoxAdapter(
      child: ListButton(
        text: Text.rich(context.l10n.historyDetailScreenReportIssueCta.toTextSpan(context)),
        onPressed: () => PlaceholderScreen.showGeneric(context),
        dividerSide: side,
      ),
    );
  }
}
