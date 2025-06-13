import 'package:bloc_test/bloc_test.dart';
import 'package:collection/collection.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/feature/history/overview/bloc/history_overview_bloc.dart';
import 'package:wallet/src/feature/history/overview/history_overview_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

class MockHistoryOverviewBloc extends MockBloc<HistoryOverviewEvent, HistoryOverviewState>
    implements HistoryOverviewBloc {}

void main() {
  final historyOverviewLoadSuccessMock = HistoryOverviewLoadSuccess(
    [
      WalletMockData.disclosureEvent,
      WalletMockData.disclosureEvent,
      WalletMockData.signEvent,
      WalletMockData.issuanceEvent,
      WalletMockData.renewEvent,
    ].sortedBy((card) => card.dateTime).reversed.toList() /* sorting is normally handled by repo layer */,
  );

  group('goldens', () {
    testGoldens('HistoryOverviewLoadSuccess light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryOverviewScreen().withState<HistoryOverviewBloc, HistoryOverviewState>(
          MockHistoryOverviewBloc(),
          historyOverviewLoadSuccessMock,
        ),
      );
      await screenMatchesGolden('success.light');
    });

    testGoldens('HistoryOverviewLoadSuccess dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryOverviewScreen().withState<HistoryOverviewBloc, HistoryOverviewState>(
          MockHistoryOverviewBloc(),
          historyOverviewLoadSuccessMock,
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('success.dark');
    });

    testGoldens('HistoryOverviewLoadInProgress light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryOverviewScreen().withState<HistoryOverviewBloc, HistoryOverviewState>(
          MockHistoryOverviewBloc(),
          const HistoryOverviewLoadInProgress(),
        ),
      );
      await screenMatchesGolden('loading.light');
    });

    testGoldens('HistoryOverviewLoadFailure light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryOverviewScreen().withState<HistoryOverviewBloc, HistoryOverviewState>(
          MockHistoryOverviewBloc(),
          const HistoryOverviewLoadFailure(error: GenericError('', sourceError: 'test')),
        ),
      );
      await screenMatchesGolden('error.light');
    });
  });

  group('widgets', () {
    testWidgets('OperationAttribute renders the card title', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryOverviewScreen().withState<HistoryOverviewBloc, HistoryOverviewState>(
          MockHistoryOverviewBloc(),
          HistoryOverviewLoadSuccess([
            WalletMockData.issuanceEvent,
          ]),
        ),
      );

      expect(find.text(WalletMockData.issuanceEvent.card.title.testValue), findsOneWidget);
    });

    testWidgets('SignAttribute renders the organization title', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryOverviewScreen().withState<HistoryOverviewBloc, HistoryOverviewState>(
          MockHistoryOverviewBloc(),
          HistoryOverviewLoadSuccess([
            WalletMockData.signEvent,
          ]),
        ),
      );

      // Sign attribute renders the title of the organization
      expect(find.text(WalletMockData.organization.displayName.testValue), findsOneWidget);
    });

    testWidgets('InteractionAttribute renders the organization title', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryOverviewScreen().withState<HistoryOverviewBloc, HistoryOverviewState>(
          MockHistoryOverviewBloc(),
          HistoryOverviewLoadSuccess([
            WalletMockData.disclosureEvent,
          ]),
        ),
      );

      // Interaction attribute renders the title of the organization
      expect(find.text(WalletMockData.organization.displayName.testValue), findsOneWidget);
    });

    testWidgets('HistoryOverviewLoadFailure shows error description and retry cta', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryOverviewScreen().withState<HistoryOverviewBloc, HistoryOverviewState>(
          MockHistoryOverviewBloc(),
          const HistoryOverviewLoadFailure(error: GenericError('', sourceError: 'test')),
        ),
      );

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.errorScreenGenericDescription), findsOneWidget);
      expect(find.text(l10n.generalRetry), findsOneWidget);
    });

    testWidgets('Disclosure events displays type, verifier name, logo, and timestamp', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryOverviewScreen().withState<HistoryOverviewBloc, HistoryOverviewState>(
          MockHistoryOverviewBloc(),
          HistoryOverviewLoadSuccess([
            WalletMockData.disclosureEvent,
            WalletMockData.loginEvent,
          ]),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text('February 1'), findsOneWidget);
      expect(find.text('March 1'), findsOneWidget);
      expect(find.text(l10n.cardHistoryDisclosureSuccess), findsOneWidget);
      expect(find.text(l10n.cardHistoryLoginSuccess), findsOneWidget);
      expect(find.text(WalletMockData.organization.displayName.testValue), findsExactly(2));
      expect(find.byType(Image), findsExactly(2));
    });

    testWidgets('Rejected sharing event shows user aborted indication', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryOverviewScreen().withState<HistoryOverviewBloc, HistoryOverviewState>(
          MockHistoryOverviewBloc(),
          HistoryOverviewLoadSuccess([
            WalletMockData.cancelledDisclosureEvent,
          ]),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.cardHistoryDisclosureCancelled), findsOneWidget);
    });

    testWidgets('Failed sharing event shows transaction failure indication', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryOverviewScreen().withState<HistoryOverviewBloc, HistoryOverviewState>(
          MockHistoryOverviewBloc(),
          HistoryOverviewLoadSuccess([
            WalletMockData.failedDisclosureEvent,
          ]),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.cardHistoryDisclosureError), findsOneWidget);
    });

    testWidgets('Issuance event displays type, timestamp, issued cards, and illustration', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryOverviewScreen().withState<HistoryOverviewBloc, HistoryOverviewState>(
          MockHistoryOverviewBloc(),
          HistoryOverviewLoadSuccess([
            WalletMockData.issuanceEvent,
          ]),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.historyDetailScreenIssuanceSuccessDescription), findsOneWidget);
      expect(find.textContaining('December 1'), findsOneWidget);
      expect(find.text(WalletMockData.issuanceEvent.card.title.testValue), findsAtLeast(1));
      expect(find.byType(Image), findsOneWidget);
    });

    testWidgets('Events are displayed in anti-chronological order and events are categorized by month', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryOverviewScreen().withState<HistoryOverviewBloc, HistoryOverviewState>(
          MockHistoryOverviewBloc(),
          HistoryOverviewLoadSuccess([
            WalletMockData.disclosureEvent,
            WalletMockData.loginEvent,
            WalletMockData.issuanceEvent,
          ]),
        ),
      );

      final firstEvent = find.textContaining('December 1');
      final secondEvent = find.textContaining('February 1');
      final thirdEvent = find.textContaining('March 1');
      expect(tester.getTopLeft(firstEvent).dy, greaterThan(tester.getTopLeft(secondEvent).dy));
      expect(tester.getTopLeft(secondEvent).dy, greaterThan(tester.getTopLeft(thirdEvent).dy));
      expect(find.text('December 2023'), findsOneWidget);
      expect(find.text('February 2024'), findsOneWidget);
      expect(find.text('March 2024'), findsOneWidget);
    });
  });
}
