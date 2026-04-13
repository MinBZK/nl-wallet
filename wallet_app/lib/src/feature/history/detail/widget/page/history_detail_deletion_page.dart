import 'package:flutter/material.dart';

import '../../../../../domain/model/event/wallet_event.dart';
import '../../../../../domain/model/organization.dart';
import '../../../../../util/extension/build_context_extension.dart';
import '../../../../../util/extension/localized_text_extension.dart';
import '../../../../../wallet_constants.dart';
import '../../../../common/builder/request_detail_common_builders.dart';
import '../../../../common/screen/placeholder_screen.dart';
import '../../../../common/widget/card/shared_attributes_card.dart';
import '../../../../common/widget/divider_side.dart';
import '../../../../common/widget/text/title_text.dart';
import '../../../../organization/detail/organization_detail_screen.dart';
import '../../../../organization/widget/organization_row.dart';
import '../history_detail_timestamp.dart';

class HistoryDetailDeletionPage extends StatelessWidget {
  final DeletionEvent event;

  const HistoryDetailDeletionPage({required this.event, super.key});

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
        _buildDeletedCard(),
        const SizedBox(height: 24),
        const Divider(),
        _buildIssuer(context, event.card.issuer),
        RequestDetailCommonBuilders.buildReportIssue(context, side: DividerSide.bottom),
        const SizedBox(height: 24),
      ],
    );
  }

  Widget _buildDeletedCard() {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: SharedAttributesCard(
        card: event.card,
        showCta: false,
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

  static String resolveTitle(BuildContext context, DeletionEvent event) {
    final cardName = event.card.title.l10nValue(context);
    return context.l10n.historyDetailScreenTitleForCardDeleted(cardName);
  }
}
