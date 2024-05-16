import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/feature/card/history/bloc/card_history_bloc.dart';
import 'package:wallet/src/feature/card/history/card_history_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../util/device_utils.dart';
import '../../../util/test_utils.dart';

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
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const CardHistoryScreen().withState<CardHistoryBloc, CardHistoryState>(
              MockCardHistoryBloc(),
              cardHistoryLoadSuccessMock,
            ),
            name: 'card data',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'success.light');
    });

    testGoldens('CardHistoryLoadSuccess dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const CardHistoryScreen().withState<CardHistoryBloc, CardHistoryState>(
              MockCardHistoryBloc(),
              cardHistoryLoadSuccessMock,
            ),
            name: 'card data',
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'success.dark');
    });

    testGoldens('CardHistoryLoadInProgress state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardHistoryScreen().withState<CardHistoryBloc, CardHistoryState>(
          MockCardHistoryBloc(),
          const CardHistoryLoadInProgress(),
        ),
      );
      await screenMatchesGolden(tester, 'loading.light');
    });

    testGoldens('CardHistoryInitial state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardHistoryScreen().withState<CardHistoryBloc, CardHistoryState>(
          MockCardHistoryBloc(),
          CardHistoryInitial(),
        ),
      );
      await screenMatchesGolden(tester, 'initial.light');
    });

    testGoldens('CardHistoryLoadFailure state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardHistoryScreen().withState<CardHistoryBloc, CardHistoryState>(
          MockCardHistoryBloc(),
          const CardHistoryLoadFailure(),
        ),
      );
      await screenMatchesGolden(tester, 'error.light');
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
            ),
            WalletEvent.issuance(
              dateTime: DateTime(2022, 12, 1),
              status: EventStatus.success,
              card: WalletMockData.card,
            ),
            WalletEvent.issuance(
              dateTime: DateTime(2022, 11, 1),
              status: EventStatus.success,
              card: WalletMockData.card,
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
      expect(appBarTitleFinder, findsNWidgets(2));
      expect(stickyJanFinder, findsOneWidget);
      expect(stickyDecFinder, findsOneWidget);
      expect(stickyNovFinder, findsOneWidget);
    });
  });
}
