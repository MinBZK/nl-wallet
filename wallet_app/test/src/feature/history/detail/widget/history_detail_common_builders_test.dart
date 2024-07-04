import 'package:collection/collection.dart';
import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/policy/policy.dart';
import 'package:wallet/src/feature/common/widget/app_image.dart';
import 'package:wallet/src/feature/common/widget/card/shared_attributes_card.dart';
import 'package:wallet/src/feature/history/detail/widget/history_detail_common_builders.dart';
import 'package:wallet/src/feature/history/detail/widget/wallet_event_status_header.dart';
import 'package:wallet/src/util/extension/localized_text_extension.dart';
import 'package:wallet/src/util/extension/wallet_event_extension.dart';
import 'package:wallet/src/util/mapper/context_mapper.dart';
import 'package:wallet/src/util/mapper/policy/policy_body_text_mapper.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mock_data.dart';
import '../../../../util/test_utils.dart';

void main() {
  testWidgets('buildStatusHeaderSliver', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      _SliverTestWrapper(
        sliverBuilder: (context) => HistoryDetailCommonBuilders.buildStatusHeaderSliver(
          context,
          WalletMockData.disclosureEvent,
        ),
      ),
    );

    final l10n = await TestUtils.englishLocalizations;
    expect(find.byType(WalletEventStatusHeader), findsOneWidget);
    expect(find.text(l10n.cardHistoryDisclosureSuccess), findsOneWidget);
  });

  testWidgets('buildPurposeSliver', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      _SliverTestWrapper(
        sliverBuilder: (context) => HistoryDetailCommonBuilders.buildPurposeSliver(
          context,
          WalletMockData.disclosureEvent,
        ),
      ),
    );

    final l10n = await TestUtils.englishLocalizations;
    expect(find.text(l10n.historyDetailScreenPurposeTitle), findsOneWidget);
    expect(find.text(WalletMockData.disclosureEvent.purpose.testValue), findsOneWidget);
  });

  testWidgets('buildAttributesSliver', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      _SliverTestWrapper(
        sliverBuilder: (context) => HistoryDetailCommonBuilders.buildAttributesSliver(
          context,
          WalletMockData.disclosureEvent,
        ),
      ),
    );

    final l10n = await TestUtils.englishLocalizations;
    expect(find.byType(SharedAttributesCard), findsNWidgets(WalletMockData.disclosureEvent.cards.length));
    expect(find.text(l10n.historyDetailScreenSharedAttributesTitle), findsOneWidget);
    final totalNrOfAttributes = WalletMockData.disclosureEvent.cards.map((card) => card.attributes).flattened.length;
    expect(find.textContaining(totalNrOfAttributes.toString()), findsOneWidget);
  });

  testWidgets('buildPolicySliver', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      _SliverTestWrapper(
        sliverBuilder: (context) => HistoryDetailCommonBuilders.buildPolicySliver(
          context,
          WalletMockData.disclosureEvent.policy,
        ),
      ).withDependency<ContextMapper<Policy, String>>((context) => PolicyBodyTextMapper()),
    );

    final l10n = await TestUtils.englishLocalizations;
    expect(find.text(l10n.historyDetailScreenTermsTitle), findsOneWidget);
    expect(find.text(l10n.historyDetailScreenTermsCta), findsOneWidget);
  });

  testWidgets('buildAboutOrganizationSliver', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      _SliverTestWrapper(
        sliverBuilder: (context) => HistoryDetailCommonBuilders.buildAboutOrganizationSliver(
          context,
          WalletMockData.disclosureEvent.relyingPartyOrIssuer,
        ),
      ),
    );

    // Make sure this CTA also renders an image (i.e. the organization logo)
    expect(find.byType(AppImage), findsOneWidget);
    final orgDisplayName = WalletMockData.disclosureEvent.relyingPartyOrIssuer.displayName.testValue;
    expect(find.textContaining(orgDisplayName), findsOneWidget);
  });

  testWidgets('buildShowDetailsSliver', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      _SliverTestWrapper(
        sliverBuilder: (context) => HistoryDetailCommonBuilders.buildShowDetailsSliver(
          context,
          WalletMockData.disclosureEvent,
        ),
      ),
    );

    final l10n = await TestUtils.englishLocalizations;
    expect(find.text(l10n.historyDetailScreenShowDetailsCta), findsOneWidget);
  });

  testWidgets('buildReportIssueSliver', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      const _SliverTestWrapper(
        sliverBuilder: HistoryDetailCommonBuilders.buildReportIssueSliver,
      ),
    );

    final l10n = await TestUtils.englishLocalizations;
    expect(find.text(l10n.historyDetailScreenReportIssueCta), findsOneWidget);
  });
}

class _SliverTestWrapper extends StatelessWidget {
  final WidgetBuilder sliverBuilder;

  const _SliverTestWrapper({required this.sliverBuilder});

  @override
  Widget build(BuildContext context) {
    return CustomScrollView(
      slivers: [sliverBuilder(context)],
    );
  }
}
