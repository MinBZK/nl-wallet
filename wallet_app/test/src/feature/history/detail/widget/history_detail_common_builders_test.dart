import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/policy/organization_policy.dart';
import 'package:wallet/src/feature/common/builder/request_detail_common_builders.dart';
import 'package:wallet/src/feature/common/widget/app_image.dart';
import 'package:wallet/src/feature/common/widget/card/shared_attributes_card.dart';
import 'package:wallet/src/feature/common/widget/divider_side.dart';
import 'package:wallet/src/feature/history/detail/widget/wallet_event_status_header.dart';
import 'package:wallet/src/util/extension/localized_text_extension.dart';
import 'package:wallet/src/util/extension/wallet_event_extension.dart';
import 'package:wallet/src/util/mapper/context_mapper.dart';
import 'package:wallet/src/util/mapper/policy/policy_body_text_mapper.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mock_data.dart';
import '../../../../test_util/test_utils.dart';

void main() {
  testWidgets('buildStatusHeaderSliver', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      _SliverTestWrapper(
        sliverBuilder: (context) => RequestDetailCommonBuilders.buildStatusHeaderSliver(
          context,
          event: WalletMockData.disclosureEvent,
          side: DividerSide.bottom,
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
        sliverBuilder: (context) => RequestDetailCommonBuilders.buildPurposeSliver(
          context,
          purpose: WalletMockData.disclosureEvent.purpose,
          side: DividerSide.bottom,
        ),
      ),
    );

    final l10n = await TestUtils.englishLocalizations;
    expect(find.text(l10n.historyDetailScreenPurposeTitle), findsOneWidget);
    expect(find.text(WalletMockData.disclosureEvent.purpose.testValue), findsOneWidget);
  });

  testWidgets('buildSharedAttributesSliver', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      _SliverTestWrapper(
        sliverBuilder: (context) => RequestDetailCommonBuilders.buildSharedAttributesSliver(
          context,
          cards: WalletMockData.disclosureEvent.cards,
          side: DividerSide.bottom,
        ),
      ),
    );

    final l10n = await TestUtils.englishLocalizations;
    expect(find.byType(SharedAttributesCard), findsNWidgets(WalletMockData.disclosureEvent.cards.length));
    expect(find.text(l10n.historyDetailScreenSharedAttributesTitle), findsOneWidget);
    final totalNrOfAttributes = WalletMockData.disclosureEvent.sharedAttributes.length;
    expect(find.textContaining(totalNrOfAttributes.toString()), findsOneWidget);
  });

  testWidgets('buildRequestedAttributesSliver', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      _SliverTestWrapper(
        sliverBuilder: (context) => RequestDetailCommonBuilders.buildRequestedAttributesSliver(
          context,
          cards: WalletMockData.disclosureEvent.cards,
          side: DividerSide.bottom,
        ),
      ),
    );

    final l10n = await TestUtils.englishLocalizations;
    expect(find.byType(SharedAttributesCard), findsNWidgets(WalletMockData.disclosureEvent.cards.length));
    expect(find.text(l10n.requestDetailsScreenAttributesTitle), findsOneWidget);
    final totalNrOfAttributes = WalletMockData.disclosureEvent.sharedAttributes.length;
    expect(find.textContaining(totalNrOfAttributes.toString()), findsOneWidget);
  });

  testWidgets('buildPolicySliver', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      _SliverTestWrapper(
        sliverBuilder: (context) => RequestDetailCommonBuilders.buildPolicySliver(
          context,
          organization: WalletMockData.disclosureEvent.relyingParty,
          policy: WalletMockData.disclosureEvent.policy,
          side: DividerSide.bottom,
        ),
      ).withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
    );

    final l10n = await TestUtils.englishLocalizations;
    expect(find.text(l10n.historyDetailScreenTermsTitle), findsOneWidget);
    expect(find.text(l10n.historyDetailScreenTermsCta), findsOneWidget);
  });

  testWidgets('buildAboutOrganizationSliver', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      _SliverTestWrapper(
        sliverBuilder: (context) => RequestDetailCommonBuilders.buildAboutOrganizationSliver(
          context,
          organization: WalletMockData.disclosureEvent.relyingPartyOrIssuer,
          side: DividerSide.bottom,
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
        sliverBuilder: (context) => RequestDetailCommonBuilders.buildShowDetailsSliver(
          context,
          event: WalletMockData.disclosureEvent,
          side: DividerSide.bottom,
        ),
      ),
    );

    final l10n = await TestUtils.englishLocalizations;
    expect(find.text(l10n.historyDetailScreenShowDetailsCta), findsOneWidget);
  });

  testWidgets('buildReportIssueSliver', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      _SliverTestWrapper(
        sliverBuilder: (c) => RequestDetailCommonBuilders.buildReportIssueSliver(c, side: DividerSide.bottom),
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
