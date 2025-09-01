import 'package:flutter/material.dart';

import '../../../../../domain/model/attribute/attribute.dart';
import '../../../../../domain/model/card/wallet_card.dart';
import '../../../../../domain/model/event/wallet_event.dart';
import '../../../../../domain/model/organization.dart';
import '../../../../../util/extension/build_context_extension.dart';
import '../../../../../util/extension/wallet_event_extension.dart';
import '../../../../../wallet_constants.dart';
import '../../../../check_attributes/check_attributes_screen.dart';
import '../../../../common/builder/request_detail_common_builders.dart';
import '../../../../common/screen/placeholder_screen.dart';
import '../../../../common/widget/card/shared_attributes_card.dart';
import '../../../../common/widget/divider_side.dart';
import '../../../../common/widget/text/title_text.dart';
import '../../../../info/info_screen.dart';
import '../../../../organization/detail/organization_detail_screen.dart';
import '../../../../organization/widget/organization_row.dart';
import '../history_detail_timestamp.dart';

class HistoryDetailIssuePage extends StatelessWidget {
  final IssuanceEvent event;

  const HistoryDetailIssuePage({required this.event, super.key});

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
        // Status header (only when not successful)
        if (!event.wasSuccess)
          RequestDetailCommonBuilders.buildStatusHeader(context, event: event, side: DividerSide.bottom),
        _buildIssuedCard(context, event.card),
        const SizedBox(height: 24),
        const Divider(),
        _buildIssuer(context, event.card.issuer),
        RequestDetailCommonBuilders.buildReportIssue(context, side: DividerSide.bottom),
        const SizedBox(height: 24),
      ],
    );
  }

  Widget _buildIssuedCard(BuildContext context, WalletCard card) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: SharedAttributesCard(
        card: card,
        onPressed: () => CheckAttributesScreen.show(
          context,
          card: card,
          onDataIncorrectPressed: () => InfoScreen.showDetailsIncorrect(context),
        ),
      ),
    );
  }

  Widget _buildIssuer(BuildContext context, Organization organization) {
    return Column(
      children: [
        OrganizationRow(
          organization: organization,
          onPressed: () => OrganizationDetailScreen.showPreloaded(
            context,
            organization,
            sharedDataWithOrganizationBefore: false,
            onReportIssuePressed: () => PlaceholderScreen.showGeneric(context),
          ),
        ),
        const Divider(),
      ],
    );
  }

  static String resolveTitle(BuildContext context, IssuanceEvent event) {
    final cardName = event.card.title.l10nValue(context);
    return event.renewed
        ? context.l10n.historyDetailScreenTitleForRenewal(cardName)
        : context.l10n.historyDetailScreenTitleForIssuance(cardName);
  }
}
