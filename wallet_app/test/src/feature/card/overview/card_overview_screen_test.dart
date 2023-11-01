import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/feature/card/overview/bloc/card_overview_bloc.dart';
import 'package:wallet/src/feature/card/overview/card_overview_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/mock_data.dart';
import '../../../util/device_utils.dart';

class MockCardOverviewBloc extends MockBloc<CardOverviewEvent, CardOverviewState> implements CardOverviewBloc {}

void main() {
  group('goldens', () {
    testGoldens('CardOverviewLoadSuccess light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const CardOverviewScreen().withState<CardOverviewBloc, CardOverviewState>(
              MockCardOverviewBloc(),
              CardOverviewLoadSuccess([WalletMockData.card, WalletMockData.altCard]),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'success.light');
    });

    testGoldens('CardOverviewLoadSuccess dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const CardOverviewScreen().withState<CardOverviewBloc, CardOverviewState>(
              MockCardOverviewBloc(),
              CardOverviewLoadSuccess([WalletMockData.card, WalletMockData.altCard]),
            ),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'success.dark');
    });

    testGoldens('CardOverviewLoadInProgress light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardOverviewScreen().withState<CardOverviewBloc, CardOverviewState>(
          MockCardOverviewBloc(),
          const CardOverviewLoadInProgress(),
        ),
      );
      await screenMatchesGolden(tester, 'loading.light');
    });

    testGoldens('CardOverviewLoadFailure light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardOverviewScreen().withState<CardOverviewBloc, CardOverviewState>(
          MockCardOverviewBloc(),
          const CardOverviewLoadFailure(),
        ),
      );
      await screenMatchesGolden(tester, 'error.light');
    });
  });

  group('widgets', () {
    testWidgets('cards are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardOverviewScreen().withState<CardOverviewBloc, CardOverviewState>(
          MockCardOverviewBloc(),
          CardOverviewLoadSuccess([WalletMockData.card, WalletMockData.altCard]),
        ),
      );

      // Validate that the widget exists
      final cardTitleFinder = find.text(WalletMockData.card.front.title.testValue);
      final altCardTitleFinder = find.text(WalletMockData.altCard.front.title.testValue);
      expect(cardTitleFinder, findsOneWidget);
      expect(altCardTitleFinder, findsOneWidget);
    });
  });
}
