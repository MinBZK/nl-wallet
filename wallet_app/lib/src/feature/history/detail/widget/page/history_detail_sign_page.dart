import 'package:flutter/material.dart';

import '../../../../../domain/model/event/wallet_event.dart';
import '../../../../../util/extension/build_context_extension.dart';
import '../../../../../util/extension/object_extension.dart';
import '../../../../../util/extension/wallet_event_extension.dart';
import '../../../../common/builder/request_detail_common_builders.dart';
import '../../../../common/widget/divider_side.dart';
import '../../../../common/widget/sliver_wallet_app_bar.dart';
import '../../../../common/widget/spacer/sliver_divider.dart';
import '../../../../common/widget/spacer/sliver_sized_box.dart';
import '../history_detail_timestamp.dart';

class HistoryDetailSignPage extends StatelessWidget {
  final SignEvent event;

  const HistoryDetailSignPage({required this.event, super.key});

  @override
  Widget build(BuildContext context) {
    return CustomScrollView(
      slivers: [
        SliverWalletAppBar(
          title: context.l10n.historyDetailScreenTitle,
          scrollController: PrimaryScrollController.maybeOf(context),
        ),
        SliverToBoxAdapter(
          child: HistoryDetailTimestamp(
            dateTime: event.dateTime,
          ),
        ),
        const SliverSizedBox(height: 24),
        const SliverDivider(),
        RequestDetailCommonBuilders.buildStatusHeaderSliver(context, event: event, side: DividerSide.bottom)
            .takeIf((_) => !event.wasSuccess),
        RequestDetailCommonBuilders.buildPolicySliver(
          context,
          organization: event.relyingParty,
          policy: event.policy,
          side: DividerSide.bottom,
        ).takeIf((_) => event.wasSuccess),
        RequestDetailCommonBuilders.buildAboutOrganizationSliver(
          context,
          organization: event.relyingParty,
          side: DividerSide.bottom,
        ),
        RequestDetailCommonBuilders.buildReportIssueSliver(context, side: DividerSide.bottom),
        const SliverSizedBox(height: 24),
      ].nonNulls.toList(),
    );
  }
}
