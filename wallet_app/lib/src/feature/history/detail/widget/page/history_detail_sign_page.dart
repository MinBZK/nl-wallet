import 'package:flutter/material.dart';

import '../../../../../domain/model/event/wallet_event.dart';
import '../../../../../util/extension/build_context_extension.dart';
import '../../../../../util/extension/wallet_event_extension.dart';
import '../../../../../wallet_constants.dart';
import '../../../../common/builder/request_detail_common_builders.dart';
import '../../../../common/widget/divider_side.dart';
import '../../../../common/widget/text/title_text.dart';
import '../history_detail_timestamp.dart';

class HistoryDetailSignPage extends StatelessWidget {
  final SignEvent event;

  const HistoryDetailSignPage({required this.event, super.key});

  @override
  Widget build(BuildContext context) {
    return ListView(
      children: [
        Padding(
          padding: kDefaultTitlePadding,
          child: TitleText(context.l10n.historyDetailScreenTitle),
        ),
        HistoryDetailTimestamp(dateTime: event.dateTime),
        const SizedBox(height: 24),
        const Divider(),
        if (!event.wasSuccess)
          RequestDetailCommonBuilders.buildStatusHeader(context, event: event, side: DividerSide.bottom),
        if (event.wasSuccess)
          RequestDetailCommonBuilders.buildPolicy(
            context,
            organization: event.relyingParty,
            policy: event.policy,
            side: DividerSide.bottom,
          ),
        RequestDetailCommonBuilders.buildAboutOrganization(
          context,
          organization: event.relyingParty,
          side: DividerSide.bottom,
        ),
        RequestDetailCommonBuilders.buildReportIssue(context, side: DividerSide.bottom),
        const SizedBox(height: 24),
      ],
    );
  }
}
