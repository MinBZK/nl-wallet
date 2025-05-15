import 'package:collection/collection.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/model/event/wallet_event.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/organization_policy.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../theme/base_wallet_theme.dart';
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
import '../widget/button/list_button.dart';
import '../widget/card/shared_attributes_card.dart';
import '../widget/spacer/sliver_divider.dart';
import '../widget/spacer/sliver_sized_box.dart';
import '../widget/text/body_text.dart';
import '../widget/text/title_text.dart';

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
    return SliverMainAxisGroup(
      slivers: [
        if (side.top) const SliverDivider(),
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
                Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    TitleText(
                      context.l10n.historyDetailScreenPurposeTitle,
                      style: BaseWalletTheme.headlineExtraSmallTextStyle,
                    ),
                    const SizedBox(height: 8),
                    BodyText(purpose.l10nValue(context)),
                  ],
                ),
              ],
            ),
          ),
        ),
        if (side.bottom) const SliverDivider(),
      ],
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
                Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    TitleText(
                      title,
                      style: BaseWalletTheme.headlineExtraSmallTextStyle,
                    ),
                    const SizedBox(height: 8),
                    BodyText(
                      subtitle,
                    ),
                  ],
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
    return SliverMainAxisGroup(
      slivers: [
        if (side.top) const SliverDivider(),
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
                Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    TitleText(
                      context.l10n.historyDetailScreenTermsTitle,
                      style: BaseWalletTheme.headlineExtraSmallTextStyle,
                    ),
                    const SizedBox(height: 8),
                    BodyText(
                      policyTextMapper.map(context, orgPolicy),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
        SliverToBoxAdapter(
          child: ListButton(
            text: Text.rich(context.l10n.historyDetailScreenTermsCta.toTextSpan(context)),
            dividerSide: DividerSide.none,
            onPressed: () => PolicyScreen.show(context, organization, policy),
          ),
        ),
        if (side.bottom) const SliverDivider(),
      ],
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
