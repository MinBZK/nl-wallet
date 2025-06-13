import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
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
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

class MockHistoryDetailBloc extends MockBloc<HistoryDetailEvent, HistoryDetailState> implements HistoryDetailBloc {}

const _kVeryTallScreen = Size(400, 3000);

void main() {
  group('goldens', () {
    testGoldens('HistoryDetailLoadSuccess light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          HistoryDetailLoadSuccess(WalletMockData.disclosureEvent, [WalletMockData.card]),
        ),
        providers: [
          RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(
            create: (c) => PolicyBodyTextMapper(),
          ),
        ],
      );
      await screenMatchesGolden('success.light');
    });

    testGoldens('HistoryDetailLoadSuccess issuance - light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          HistoryDetailLoadSuccess(WalletMockData.issuanceEvent, [WalletMockData.card]),
        ),
        providers: [
          RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(
            create: (c) => PolicyBodyTextMapper(),
          ),
        ],
      );
      await screenMatchesGolden('success.issuance.light');
    });

    testGoldens('HistoryDetailLoadSuccess renewal - light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          HistoryDetailLoadSuccess(WalletMockData.renewEvent, [WalletMockData.card]),
        ),
        providers: [
          RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(
            create: (c) => PolicyBodyTextMapper(),
          ),
        ],
      );
      await screenMatchesGolden('success.renewal.light');
    });

    testGoldens('HistoryDetailLoadSuccess dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          HistoryDetailLoadSuccess(WalletMockData.disclosureEvent, [WalletMockData.card]),
        ),
        brightness: Brightness.dark,
        providers: [
          RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(
            create: (c) => PolicyBodyTextMapper(),
          ),
        ],
      );
      await screenMatchesGolden('success.dark');
    });

    testGoldens('HistoryDetailLoadSuccess - dark landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          HistoryDetailLoadSuccess(WalletMockData.disclosureEvent, [WalletMockData.card]),
        ),
        brightness: Brightness.dark,
        providers: [
          RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(
            create: (c) => PolicyBodyTextMapper(),
          ),
        ],
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('success.dark.landscape');
    });

    testGoldens('HistoryDetailLoadInProgress light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          const HistoryDetailLoadInProgress(),
        ),
      );
      await screenMatchesGolden('loading.light');
    });

    testGoldens('Error state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          const HistoryDetailLoadFailure(),
        ),
      );
      await screenMatchesGolden('loading.error.light');
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
      await screenMatchesGolden('cancelled.light');
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
      await screenMatchesGolden('cancelled.dark');
    });

    testGoldens('Disclose error - some attributes possibly shared', (tester) async {
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
      await screenMatchesGolden('disclose.error.light');
    });

    testGoldens('Disclose error - no attributes shared', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          HistoryDetailLoadSuccess(
            WalletMockData.failedDisclosureEventNothingShared,
            const [],
          ),
        ),
        providers: [
          RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(
            create: (c) => PolicyBodyTextMapper(),
          ),
        ],
      );
      await screenMatchesGolden('disclose.error.nothing_shared.light');
    });

    testGoldens('Disclose error - dark - some attributes possibly shared', (tester) async {
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
      await screenMatchesGolden('disclose.error.dark');
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
      await screenMatchesGolden('login.error.light');
    });

    testGoldens('Login error - nothing shared', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          HistoryDetailLoadSuccess(WalletMockData.failedLoginEventNothingShared, [WalletMockData.card]),
        ),
        providers: [
          RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(
            create: (c) => PolicyBodyTextMapper(),
          ),
        ],
      );
      await screenMatchesGolden('login.error.nothing_shared.light');
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
      final l10n = await TestUtils.englishLocalizations;
      expect(find.byType(HistoryDetailIssuePage), findsOneWidget);
      expect(find.textContaining(WalletMockData.organization.displayName.testValue), findsOneWidget);
      final count = WalletMockData.issuanceEvent.sharedAttributes.length;
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
