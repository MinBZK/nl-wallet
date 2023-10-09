import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/card/data/bloc/card_data_bloc.dart';
import 'package:wallet/src/feature/card/data/card_data_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/mock_data.dart';
import '../../../util/device_utils.dart';

class MockCardDataBloc extends MockBloc<CardDataEvent, CardDataState> implements CardDataBloc {}

void main() {
  group('goldens', () {
    testGoldens('CardDataLoadSuccess Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const CardDataScreen(cardTitle: 'Title').withState<CardDataBloc, CardDataState>(
              MockCardDataBloc(),
              const CardDataLoadSuccess(WalletMockData.card),
            ),
            name: 'card data',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'success.light');
    });

    testGoldens('CardDataLoadSuccess Dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const CardDataScreen(cardTitle: 'Title').withState<CardDataBloc, CardDataState>(
              MockCardDataBloc(),
              const CardDataLoadSuccess(WalletMockData.card),
            ),
            name: 'card data',
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'success.dark');
    });

    testGoldens('CardDataInitial state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Card Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          CardDataInitial(),
        ),
      );
      await screenMatchesGolden(tester, 'initial.light');
    });

    testGoldens('CardDataLoadInProgress state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Card Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          const CardDataLoadInProgress(),
        ),
      );
      await screenMatchesGolden(tester, 'loading.light');
    });

    testGoldens('Privacy Banner Sheet', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Card Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          const CardDataLoadSuccess(WalletMockData.card),
        ),
      );
      await tester.tap(find.byKey(kPrivacyBannerKey));
      await tester.pumpAndSettle();
      await screenMatchesGolden(tester, 'privacy_sheet.light');
    });

    testGoldens('CardDataLoadFailure state', (tester) async {
      await tester.pumpDeviceBuilderWithAppWrapper(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const CardDataScreen(cardTitle: 'Card Title').withState<CardDataBloc, CardDataState>(
              MockCardDataBloc(),
              const CardDataLoadFailure(),
            ),
          ),
      );
      await screenMatchesGolden(tester, 'error.light');
    });
  });

  group('widgets', () {
    testWidgets('card title is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Card Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          const CardDataLoadSuccess(WalletMockData.card),
        ),
      );

      // Validate that the widget exists
      final titleFinder = find.text('Sample Card');
      final labelFinder = find.text(WalletMockData.textDataAttribute.label);
      final valueFinder = find.text(WalletMockData.textDataAttribute.value);
      expect(titleFinder, findsOneWidget);
      expect(labelFinder, findsNWidgets(2));
      expect(valueFinder, findsOneWidget);
    });
  });
}
