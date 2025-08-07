import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/feature/card/history/bloc/card_history_bloc.dart';
import 'package:wallet/src/feature/card/history/card_history_screen.dart';
import 'package:wallet/src/feature/common/widget/centered_loading_indicator.dart';
import 'package:wallet/src/navigation/wallet_routes.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

class MockCardHistoryBloc extends MockBloc<CardHistoryEvent, CardHistoryState> implements CardHistoryBloc {}

void main() {
  final cardHistoryLoadSuccessMock = CardHistoryLoadSuccess(
    WalletMockData.card,
    [
      WalletMockData.disclosureEvent,
      WalletMockData.issuanceEvent,
      WalletMockData.signEvent,
    ],
  );

  group('goldens', () {
    testGoldens('CardHistoryLoadSuccess light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardHistoryScreen().withState<CardHistoryBloc, CardHistoryState>(
          MockCardHistoryBloc(),
          cardHistoryLoadSuccessMock,
        ),
      );

      await screenMatchesGolden('success.light');
    });

    testGoldens('CardHistoryLoadSuccess dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardHistoryScreen().withState<CardHistoryBloc, CardHistoryState>(
          MockCardHistoryBloc(),
          cardHistoryLoadSuccessMock,
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('success.dark');
    });

    testGoldens('CardHistoryLoadInProgress state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardHistoryScreen().withState<CardHistoryBloc, CardHistoryState>(
          MockCardHistoryBloc(),
          const CardHistoryLoadInProgress(),
        ),
      );
      await screenMatchesGolden('loading.light');
    });

    testGoldens('CardHistoryInitial state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardHistoryScreen().withState<CardHistoryBloc, CardHistoryState>(
          MockCardHistoryBloc(),
          CardHistoryInitial(),
        ),
      );
      await screenMatchesGolden('initial.light');
    });

    testGoldens('CardHistoryLoadFailure state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardHistoryScreen().withState<CardHistoryBloc, CardHistoryState>(
          MockCardHistoryBloc(),
          const CardHistoryLoadFailure(),
        ),
      );
      await screenMatchesGolden('error.light');
    });
  });

  group('widgets', () {
    testWidgets('sticky headers are shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardHistoryScreen().withState<CardHistoryBloc, CardHistoryState>(
          MockCardHistoryBloc(),
          CardHistoryLoadSuccess(WalletMockData.card, [
            WalletEvent.issuance(
              dateTime: DateTime(2023, 1, 1),
              status: EventStatus.success,
              card: WalletMockData.card,
              renewed: false,
            ),
            WalletEvent.issuance(
              dateTime: DateTime(2022, 12, 1),
              status: EventStatus.success,
              card: WalletMockData.card,
              renewed: false,
            ),
            WalletEvent.issuance(
              dateTime: DateTime(2022, 11, 1),
              status: EventStatus.success,
              card: WalletMockData.card,
              renewed: false,
            ),
          ]),
        ),
      );

      // Validate that the widget exists
      final l10n = await TestUtils.englishLocalizations;
      final appBarTitleFinder = find.text(l10n.cardHistoryScreenTitle);
      final stickyJanFinder = find.text('January 2023');
      final stickyDecFinder = find.text('December 2022');
      final stickyNovFinder = find.text('November 2022');
      expect(appBarTitleFinder, findsOneWidget);
      expect(stickyJanFinder, findsOneWidget);
      expect(stickyDecFinder, findsOneWidget);
      expect(stickyNovFinder, findsOneWidget);
    });

    testWidgets('loading is rendered as expected', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardHistoryScreen().withState<CardHistoryBloc, CardHistoryState>(
          MockCardHistoryBloc(),
          const CardHistoryLoadInProgress(),
        ),
      );

      expect(find.byType(CenteredLoadingIndicator), findsOneWidget);
    });

    testWidgets('error is rendered as expected, with error description and retry cta', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardHistoryScreen().withState<CardHistoryBloc, CardHistoryState>(
          MockCardHistoryBloc(),
          const CardHistoryLoadFailure(),
        ),
      );

      // Validate that the widget exists
      final l10n = await TestUtils.englishLocalizations;
      final retryCtaFinder = find.text(l10n.generalRetry);
      final descriptionFinder = find.text(l10n.errorScreenGenericDescription);
      expect(retryCtaFinder, findsOneWidget);
      expect(descriptionFinder, findsOneWidget);
    });

    testWidgets('onRowPressed triggers navigation event', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardHistoryScreen().withState<CardHistoryBloc, CardHistoryState>(
          MockCardHistoryBloc(),
          CardHistoryLoadSuccess(WalletMockData.card, [
            WalletEvent.issuance(
              dateTime: DateTime(2023, 1, 1),
              status: EventStatus.success,
              card: WalletMockData.card,
              renewed: false,
            ),
          ]),
        ),
      );

      // Tap the card row
      final rowFinder = find.text(WalletMockData.card.title.testValue);
      expect(rowFinder, findsOneWidget);
      await tester.tap(rowFinder);
      await tester.pumpAndSettle();

      // Verify navigation occurred
      final newPageFinder = find.text(WalletRoutes.historyDetailRoute);
      expect(newPageFinder, findsOneWidget);
    });
  });
}
