import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/policy/organization_policy.dart';
import 'package:wallet/src/feature/history/detail/bloc/history_detail_bloc.dart';
import 'package:wallet/src/feature/history/detail/history_detail_screen.dart';
import 'package:wallet/src/feature/history/detail/widget/page/history_detail_disclose_page.dart';
import 'package:wallet/src/feature/history/detail/widget/page/history_detail_issue_page.dart';
import 'package:wallet/src/feature/history/detail/widget/page/history_detail_login_page.dart';
import 'package:wallet/src/feature/history/detail/widget/page/history_detail_sign_page.dart';
import 'package:wallet/src/util/mapper/context_mapper.dart';
import 'package:wallet/src/util/mapper/policy/policy_body_text_mapper.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../util/device_utils.dart';
import '../../../util/test_utils.dart';

class MockHistoryDetailBloc extends MockBloc<HistoryDetailEvent, HistoryDetailState> implements HistoryDetailBloc {}

const _kVeryTallScreen = Size(400, 3000);

void main() {
  group('goldens', () {
    testGoldens('HistoryDetailLoadSuccess light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
              MockHistoryDetailBloc(),
              HistoryDetailLoadSuccess(WalletMockData.disclosureEvent, [WalletMockData.card]),
            ),
          ),
        wrapper: walletAppWrapper(
          providers: [
            RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(
              create: (c) => PolicyBodyTextMapper(),
            ),
          ],
        ),
      );
      await screenMatchesGolden(tester, 'success.light');
    });

    testGoldens('HistoryDetailLoadSuccess dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
              MockHistoryDetailBloc(),
              HistoryDetailLoadSuccess(WalletMockData.disclosureEvent, [WalletMockData.card]),
            ),
          ),
        wrapper: walletAppWrapper(
          brightness: Brightness.dark,
          providers: [
            RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(
              create: (c) => PolicyBodyTextMapper(),
            ),
          ],
        ),
      );
      await screenMatchesGolden(tester, 'success.dark');
    });

    testGoldens('HistoryDetailLoadInProgress light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          const HistoryDetailLoadInProgress(),
        ),
      );
      await screenMatchesGolden(tester, 'loading.light');
    });

    testGoldens('Error state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          const HistoryDetailLoadFailure(),
        ),
      );
      await screenMatchesGolden(tester, 'loading.error.light');
    });

    testGoldens('Disclose cancelled', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          HistoryDetailLoadSuccess(WalletMockData.cancelledDisclosureEvent, [WalletMockData.card]),
        ),
        providers: [
          RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(
            create: (c) => PolicyBodyTextMapper(),
          ),
        ],
      );
      await screenMatchesGolden(tester, 'cancelled.light');
    });

    testGoldens('Disclose cancelled - dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          HistoryDetailLoadSuccess(WalletMockData.cancelledDisclosureEvent, [WalletMockData.card]),
        ),
        brightness: Brightness.dark,
        providers: [
          RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(
            create: (c) => PolicyBodyTextMapper(),
          ),
        ],
      );
      await screenMatchesGolden(tester, 'cancelled.dark');
    });

    testGoldens('Disclose error', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          HistoryDetailLoadSuccess(WalletMockData.failedDisclosureEvent, [WalletMockData.card]),
        ),
        providers: [
          RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(
            create: (c) => PolicyBodyTextMapper(),
          ),
        ],
      );
      await screenMatchesGolden(tester, 'disclose.error.light');
    });

    testGoldens('Disclose error - dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          HistoryDetailLoadSuccess(WalletMockData.failedDisclosureEvent, [WalletMockData.card]),
        ),
        brightness: Brightness.dark,
        providers: [
          RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(
            create: (c) => PolicyBodyTextMapper(),
          ),
        ],
      );
      await screenMatchesGolden(tester, 'disclose.error.dark');
    });

    testGoldens('Login error', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          HistoryDetailLoadSuccess(WalletMockData.failedLoginEvent, [WalletMockData.card]),
        ),
        providers: [
          RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(
            create: (c) => PolicyBodyTextMapper(),
          ),
        ],
      );
      await screenMatchesGolden(tester, 'login.error.light');
    });
  });

  group('widgets', () {
    testWidgets('Disclose event is rendered with DisclosePage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen()
            .withState<HistoryDetailBloc, HistoryDetailState>(
              MockHistoryDetailBloc(),
              HistoryDetailLoadSuccess(WalletMockData.disclosureEvent, [WalletMockData.card]),
            )
            .withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
        surfaceSize: _kVeryTallScreen,
      );
      final l10n = await TestUtils.englishLocalizations;
      //sharedAttributesCardTitle
      expect(find.byType(HistoryDetailDisclosePage), findsOneWidget);
      expect(find.textContaining(WalletMockData.organization.displayName.testValue), findsAtLeast(1));
      expect(find.text(WalletMockData.disclosureEvent.purpose.testValue), findsOneWidget);
      final count = WalletMockData.disclosureEvent.cards.first.attributes.length.toString();
      expect(find.text('$count from ${WalletMockData.card.title.testValue}'), findsOneWidget);
      expect(find.text(WalletMockData.disclosureEvent.cards.first.attributes.first.label.testValue), findsOneWidget);
      expect(find.text('1 March 2024, 00:00'), findsOneWidget);
      expect(find.textContaining('will store your data for', findRichText: true), findsOneWidget);
      final scrollableFinder = find.byType(Scrollable);
      await tester.scrollUntilVisible(
        find.text(l10n.disclosureStopSheetReportIssueCta),
        500,
        scrollable: scrollableFinder,
      );
      expect(find.textContaining(l10n.disclosureStopSheetReportIssueCta), findsOneWidget);
      expect(find.text(l10n.generalBottomBackCta), findsOneWidget);
    });

    testWidgets('Issuance event is rendered with IssuePage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          HistoryDetailLoadSuccess(WalletMockData.issuanceEvent, [WalletMockData.card]),
        ),
      );
      debugDumpApp();
      final l10n = await TestUtils.englishLocalizations;
      expect(find.byType(HistoryDetailIssuePage), findsOneWidget);
      expect(find.textContaining(WalletMockData.organization.displayName.testValue), findsOneWidget);
      final count = WalletMockData.issuanceEvent.attributes.length;
      expect(find.text('$count from ${WalletMockData.card.title.testValue}'), findsOneWidget);
      expect(find.text('1 December 2023, 00:00'), findsOneWidget);
      expect(find.textContaining(l10n.disclosureStopSheetReportIssueCta), findsOneWidget);
      expect(find.text(l10n.generalBottomBackCta), findsOneWidget);
    });

    testWidgets('Login event is rendered with LoginPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen()
            .withState<HistoryDetailBloc, HistoryDetailState>(
              MockHistoryDetailBloc(),
              HistoryDetailLoadSuccess(WalletMockData.loginEvent, [WalletMockData.card]),
            )
            .withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
        surfaceSize: _kVeryTallScreen,
      );

      debugDumpApp();
      final l10n = await TestUtils.englishLocalizations;
      //sharedAttributesCardTitle
      expect(find.byType(HistoryDetailLoginPage), findsOneWidget);
      expect(find.textContaining(WalletMockData.organization.displayName.testValue), findsAtLeast(1));
      expect(find.text(WalletMockData.loginEvent.purpose.testValue), findsOneWidget);
      final count = WalletMockData.loginEvent.cards.first.attributes.length.toString();
      expect(find.text('$count from ${WalletMockData.card.title.testValue}'), findsOneWidget);
      expect(find.text(WalletMockData.loginEvent.cards.first.attributes.first.label.testValue), findsOneWidget);
      expect(find.text('1 February 2024, 00:00'), findsOneWidget);
      expect(find.textContaining('will store your data for', findRichText: true), findsOneWidget);
      final scrollableFinder = find.byType(Scrollable);
      await tester.scrollUntilVisible(
        find.text(l10n.disclosureStopSheetReportIssueCta),
        500,
        scrollable: scrollableFinder,
      );
      expect(find.textContaining(l10n.disclosureStopSheetReportIssueCta), findsOneWidget);
      expect(find.text(l10n.generalBottomBackCta), findsOneWidget);
    });

    testWidgets('Sign event is rendered with SignPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen()
            .withState<HistoryDetailBloc, HistoryDetailState>(
              MockHistoryDetailBloc(),
              HistoryDetailLoadSuccess(WalletMockData.signEvent, [WalletMockData.card]),
            )
            .withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
      );

      expect(find.byType(HistoryDetailSignPage), findsOneWidget);
    });

    testWidgets('Error screen is rendered for HistoryDetailLoadFailure', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen()
            .withState<HistoryDetailBloc, HistoryDetailState>(
              MockHistoryDetailBloc(),
              const HistoryDetailLoadFailure(),
            )
            .withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
      );

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.historyDetailScreenTitle), findsAtLeast(1));
      expect(find.text(l10n.errorScreenGenericDescription), findsOneWidget);
      expect(find.text(l10n.generalRetry), findsOneWidget); // CTA to retry is visible
    });
  });
}
