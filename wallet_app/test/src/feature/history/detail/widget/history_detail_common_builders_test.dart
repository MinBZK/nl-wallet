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
  testWidgets('buildStatusHeader', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      _WidgetTestWrapper(
        widgetBuilder: (context) => RequestDetailCommonBuilders.buildStatusHeader(
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

  testWidgets('buildPurpose', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      _WidgetTestWrapper(
        widgetBuilder: (context) => RequestDetailCommonBuilders.buildPurpose(
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

  testWidgets('buildSharedAttributes', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      _WidgetTestWrapper(
        widgetBuilder: (context) => RequestDetailCommonBuilders.buildSharedAttributes(
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

  testWidgets('buildRequestedAttributes', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      _WidgetTestWrapper(
        widgetBuilder: (context) => RequestDetailCommonBuilders.buildRequestedAttributes(
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

  testWidgets('buildPolicy', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      _WidgetTestWrapper(
        widgetBuilder: (context) => RequestDetailCommonBuilders.buildPolicy(
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

  testWidgets('buildAboutOrganization', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      _WidgetTestWrapper(
        widgetBuilder: (context) => RequestDetailCommonBuilders.buildAboutOrganization(
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

  testWidgets('buildShowDetails', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      _WidgetTestWrapper(
        widgetBuilder: (context) => RequestDetailCommonBuilders.buildShowDetails(
          context,
          event: WalletMockData.disclosureEvent,
          side: DividerSide.bottom,
        ),
      ),
    );

    final l10n = await TestUtils.englishLocalizations;
    expect(find.text(l10n.historyDetailScreenShowDetailsCta), findsOneWidget);
  });

  testWidgets('buildReportIssue', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      _WidgetTestWrapper(
        widgetBuilder: (c) => RequestDetailCommonBuilders.buildReportIssue(c, side: DividerSide.bottom),
      ),
    );

    final l10n = await TestUtils.englishLocalizations;
    expect(find.text(l10n.historyDetailScreenReportIssueCta), findsOneWidget);
  });
}

class _WidgetTestWrapper extends StatelessWidget {
  final WidgetBuilder widgetBuilder;

  const _WidgetTestWrapper({required this.widgetBuilder});

  @override
  Widget build(BuildContext context) => widgetBuilder(context);
}
