import 'package:flutter/material.dart';

import '../../../../../domain/model/attribute/attribute.dart';
import '../../../../../domain/model/card/wallet_card.dart';
import '../../../../../domain/model/event/wallet_event.dart';
import '../../../../../domain/model/organization.dart';
import '../../../../../util/extension/build_context_extension.dart';
import '../../../../../util/extension/object_extension.dart';
import '../../../../../util/extension/wallet_event_extension.dart';
import '../../../../check_attributes/check_attributes_screen.dart';
import '../../../../common/builder/request_detail_common_builders.dart';
import '../../../../common/screen/placeholder_screen.dart';
import '../../../../common/widget/card/shared_attributes_card.dart';
import '../../../../common/widget/divider_side.dart';
import '../../../../common/widget/sliver_wallet_app_bar.dart';
import '../../../../common/widget/spacer/sliver_divider.dart';
import '../../../../common/widget/spacer/sliver_sized_box.dart';
import '../../../../info/info_screen.dart';
import '../../../../organization/detail/organization_detail_screen.dart';
import '../../../../organization/widget/organization_row.dart';
import '../history_detail_timestamp.dart';

class HistoryDetailIssuePage extends StatelessWidget {
  final IssuanceEvent event;

  const HistoryDetailIssuePage({required this.event, super.key});

  @override
  Widget build(BuildContext context) {
    final cardName = event.card.title.l10nValue(context);
    final title = event.renewed
        ? context.l10n.historyDetailScreenTitleForRenewal(cardName)
        : context.l10n.historyDetailScreenTitleForIssuance(cardName);
    return CustomScrollView(
      slivers: [
        SliverWalletAppBar(
          title: title,
          scrollController: PrimaryScrollController.maybeOf(context),
        ),
        SliverToBoxAdapter(
          child: HistoryDetailTimestamp(
            dateTime: event.dateTime,
          ),
        ),
        const SliverSizedBox(height: 24),
        RequestDetailCommonBuilders.buildStatusHeaderSliver(context, event: event, side: DividerSide.bottom)
            .takeIf((_) => !event.wasSuccess),
        _buildIssuedCardSliver(context, event.card),
        const SliverSizedBox(height: 24),
        const SliverDivider(),
        _buildIssuerSliver(context, event.card.issuer),
        RequestDetailCommonBuilders.buildReportIssueSliver(context, side: DividerSide.bottom),
        const SliverSizedBox(height: 24),
      ].nonNulls.toList(),
    );
  }

  Widget _buildIssuedCardSliver(BuildContext context, WalletCard card) {
    return SliverToBoxAdapter(
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16),
        child: SharedAttributesCard(
          card: card,
          onPressed: () => CheckAttributesScreen.show(
            context,
            card: card,
            onDataIncorrectPressed: () => InfoScreen.showDetailsIncorrect(context),
          ),
        ),
      ),
    );
  }

  Widget _buildIssuerSliver(BuildContext context, Organization organization) {
    return SliverMainAxisGroup(
      slivers: [
        SliverToBoxAdapter(
          child: OrganizationRow(
            organization: organization,
            onPressed: () => OrganizationDetailScreen.showPreloaded(
              context,
              organization,
              sharedDataWithOrganizationBefore: false,
              onReportIssuePressed: () => PlaceholderScreen.showGeneric(context),
            ),
          ),
        ),
        const SliverDivider(),
      ],
    );
  }
}
