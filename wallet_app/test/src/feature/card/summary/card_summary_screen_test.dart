import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/wallet_card_summary.dart';
import 'package:wallet/src/feature/card/summary/bloc/card_summary_bloc.dart';
import 'package:wallet/src/feature/card/summary/card_summary_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/mock_data.dart';
import '../../../util/device_utils.dart';

class MockCardSummaryBloc extends MockBloc<CardSummaryEvent, CardSummaryState> implements CardSummaryBloc {}

void main() {
  final cardSummaryLoadSuccessMock = CardSummaryLoadSuccess(
    WalletCardSummary(
      card: WalletMockData.card,
      issuer: WalletMockData.organization,
      latestIssuedOperation: WalletMockData.operationTimelineAttribute,
      latestSuccessInteraction: WalletMockData.interactionTimelineAttribute,
    ),
  );

  group('goldens', () {
    testGoldens('CardSummaryLoadSuccess light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: CardSummaryScreen(
              cardTitle: WalletMockData.card.front.title,
            ).withState<CardSummaryBloc, CardSummaryState>(
              MockCardSummaryBloc(),
              cardSummaryLoadSuccessMock,
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'success.light');
    });

    testGoldens('CardSummaryLoadSuccess dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: CardSummaryScreen(
              cardTitle: WalletMockData.card.front.title,
            ).withState<CardSummaryBloc, CardSummaryState>(
              MockCardSummaryBloc(),
              cardSummaryLoadSuccessMock,
            ),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'success.dark');
    });

    testGoldens('CardSummaryLoadInProgress light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardSummaryScreen(
          cardTitle: WalletMockData.card.front.title,
        ).withState<CardSummaryBloc, CardSummaryState>(
          MockCardSummaryBloc(),
          const CardSummaryLoadInProgress(),
        ),
      );
      await screenMatchesGolden(tester, 'loading.light');
    });

    testGoldens('CardSummaryLoadFailure light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardSummaryScreen(
          cardTitle: WalletMockData.card.front.title,
        ).withState<CardSummaryBloc, CardSummaryState>(
          MockCardSummaryBloc(),
          CardSummaryLoadFailure(WalletMockData.card.id),
        ),
      );
      await screenMatchesGolden(tester, 'error.light');
    });
  });

  group('widgets', () {
    testWidgets('card is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardSummaryScreen(
          cardTitle: WalletMockData.card.front.title,
        ).withState<CardSummaryBloc, CardSummaryState>(
          MockCardSummaryBloc(),
          cardSummaryLoadSuccessMock,
        ),
      );

      // Validate that the widget exists
      final cardTitleFinder = find.text(WalletMockData.card.front.title);
      expect(cardTitleFinder, findsNWidgets(2)); // App bar title + title on card
    });
  });
}
