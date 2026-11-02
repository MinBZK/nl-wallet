import 'package:flutter/material.dart';

import '../../../../../domain/model/event/wallet_event.dart';
import '../../../../../util/extension/build_context_extension.dart';
import '../../../../../util/extension/wallet_event_extension.dart';
import '../../../../../wallet_constants.dart';
import '../../../../common/builder/request_detail_common_builders.dart';
import '../../../../common/widget/divider_side.dart';
import '../../../../common/widget/text/title_text.dart';
import '../history_detail_timestamp.dart';

/// History detail of a [DisclosureEvent], used for both the [DisclosureType.regular] and [DisclosureType.login]
/// variants, which only differ in the title that is shown (see [resolveTitle]).
class HistoryDetailDisclosurePage extends StatelessWidget {
  final DisclosureEvent event;

  const HistoryDetailDisclosurePage({required this.event, super.key});

  @override
  Widget build(BuildContext context) {
    return ListView(
      children: [
        Padding(
          padding: kDefaultTitlePadding,
          child: TitleText(resolveTitle(context, event)),
        ),
        HistoryDetailTimestamp(dateTime: event.dateTime),
        const SizedBox(height: 24),
        const Divider(),
        RequestDetailCommonBuilders.buildStatusHeader(context, event: event, side: DividerSide.bottom),
        if (event.wasSuccess) ...[
          RequestDetailCommonBuilders.buildPurpose(context, purpose: event.purpose, side: DividerSide.bottom),
          RequestDetailCommonBuilders.buildSharedAttributes(context, cards: event.cards, side: DividerSide.bottom),
          RequestDetailCommonBuilders.buildPolicy(
            context,
            organization: event.relyingParty,
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

  static String resolveTitle(BuildContext context, DisclosureEvent event) {
    final l10n = context.l10n;
    final organizationName = event.relyingParty.displayName;
    return switch ((event.type, event.status)) {
      (DisclosureType.regular, EventStatus.success) => l10n.historyDetailScreenTitleForDisclosure(organizationName),
      (DisclosureType.regular, EventStatus.cancelled) => l10n.historyDetailScreenStoppedTitleForDisclosure(
        organizationName,
      ),
      (DisclosureType.regular, EventStatus.error) when event.hasSharedAttributes =>
        l10n.historyDetailScreenErrorTitleForDisclosure(organizationName),
      (DisclosureType.regular, EventStatus.error) => l10n.historyDetailScreenErrorNoDataSharedTitleForDisclosure(
        organizationName,
      ),
      (DisclosureType.login, EventStatus.success) => l10n.historyDetailScreenTitleForLogin(organizationName),
      (DisclosureType.login, EventStatus.cancelled) => l10n.historyDetailScreenStoppedTitleForLogin(organizationName),
      (DisclosureType.login, EventStatus.error) when event.hasSharedAttributes =>
        l10n.historyDetailScreenErrorTitleForLogin(organizationName),
      (DisclosureType.login, EventStatus.error) => l10n.historyDetailScreenErrorNoDataSharedTitleForLogin(
        organizationName,
      ),
    };
  }
}
