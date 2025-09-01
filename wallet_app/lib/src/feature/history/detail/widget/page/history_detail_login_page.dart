import 'package:flutter/material.dart';

import '../../../../../domain/model/attribute/attribute.dart';
import '../../../../../domain/model/event/wallet_event.dart';
import '../../../../../util/extension/build_context_extension.dart';
import '../../../../../util/extension/wallet_event_extension.dart';
import '../../../../../wallet_constants.dart';
import '../../../../common/builder/request_detail_common_builders.dart';
import '../../../../common/widget/button/list_button.dart';
import '../../../../common/widget/divider_side.dart';
import '../../../../common/widget/text/title_text.dart';
import '../history_detail_timestamp.dart';

class HistoryDetailLoginPage extends StatelessWidget {
  final DisclosureEvent event;

  const HistoryDetailLoginPage({required this.event, super.key});

  @override
  Widget build(BuildContext context) {
    return ListView(
      children: [
        Padding(
          padding: kDefaultTitlePadding,
          child: TitleText(resolveLoginTitle(context, event)),
        ),
        HistoryDetailTimestamp(dateTime: event.dateTime),
        const SizedBox(height: 24),
        const Divider(),
        if (!event.wasSuccess)
          RequestDetailCommonBuilders.buildStatusHeader(context, event: event, side: DividerSide.bottom),
        if (event.wasSuccess) ...[
          RequestDetailCommonBuilders.buildPurpose(context, purpose: event.purpose, side: DividerSide.bottom),
          RequestDetailCommonBuilders.buildSharedAttributes(context, cards: event.cards, side: DividerSide.bottom),
          RequestDetailCommonBuilders.buildPolicy(
            context,
            organization: event.relyingParty,
            policy: event.policy,
            side: DividerSide.bottom,
          ),
        ],
        RequestDetailCommonBuilders.buildAboutOrganization(
          context,
          organization: event.relyingParty,
          side: DividerSide.bottom,
        ),
        if (!event.wasSuccess)
          RequestDetailCommonBuilders.buildShowDetails(context, event: event, side: DividerSide.bottom),
        RequestDetailCommonBuilders.buildReportIssue(context, side: DividerSide.bottom),
        const SizedBox(height: 24),
      ],
    );
  }

  static String resolveLoginTitle(BuildContext context, DisclosureEvent event) {
    final organizationName = event.relyingParty.displayName.l10nValue(context);
    return switch (event.status) {
      EventStatus.success => context.l10n.historyDetailScreenTitleForLogin(organizationName),
      EventStatus.cancelled => context.l10n.historyDetailScreenStoppedTitleForLogin(organizationName),
      EventStatus.error => event.hasSharedAttributes
          ? context.l10n.historyDetailScreenErrorTitleForLogin(organizationName)
          : context.l10n.historyDetailScreenErrorNoDataSharedTitleForLogin(organizationName),
    };
  }
}
