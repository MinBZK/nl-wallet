import 'package:flutter/material.dart';

import '../../../../../domain/model/event/wallet_event.dart';
import '../../../../../util/extension/build_context_extension.dart';
import '../../../../../util/extension/object_extension.dart';
import '../../../../../util/extension/wallet_event_extension.dart';
import '../../../../common/widget/sliver_divider.dart';
import '../../../../common/widget/sliver_sized_box.dart';
import '../../../../common/widget/sliver_wallet_app_bar.dart';
import '../history_detail_common_builders.dart';
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
        HistoryDetailCommonBuilders.buildStatusHeaderSliver(context, event).takeIf((_) => !event.wasSuccess),
        HistoryDetailCommonBuilders.buildPolicySliver(context, event.relyingParty, event.policy)
            .takeIf((_) => event.wasSuccess),
        HistoryDetailCommonBuilders.buildAboutOrganizationSliver(context, event.relyingParty),
        HistoryDetailCommonBuilders.buildReportIssueSliver(context),
        const SliverSizedBox(height: 24),
      ].nonNulls.toList(),
    );
  }
}
