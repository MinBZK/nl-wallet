import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/feature/dashboard/bloc/dashboard_bloc.dart';
import 'package:wallet/src/feature/dashboard/dashboard_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/mock_data.dart';
import '../../util/device_utils.dart';

class MockDashboardBloc extends MockBloc<DashboardEvent, DashboardState> implements DashboardBloc {}

void main() {
  group('goldens', () {
    testGoldens('DashboardLoadSuccess light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DashboardScreen().withState<DashboardBloc, DashboardState>(
              MockDashboardBloc(),
              DashboardLoadSuccess([WalletMockData.card, WalletMockData.altCard]),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'success.light');
    });

    testGoldens('DashboardLoadSuccess dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DashboardScreen().withState<DashboardBloc, DashboardState>(
              MockDashboardBloc(),
              DashboardLoadSuccess([WalletMockData.card, WalletMockData.altCard]),
            ),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'success.dark');
    });

    testGoldens('DashboardLoadInProgress light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DashboardScreen().withState<DashboardBloc, DashboardState>(
          MockDashboardBloc(),
          const DashboardLoadInProgress(),
        ),
      );
      await screenMatchesGolden(tester, 'loading.light');
    });

    testGoldens('DashboardLoadFailure light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DashboardScreen().withState<DashboardBloc, DashboardState>(
          MockDashboardBloc(),
          const DashboardLoadFailure(),
        ),
      );
      await screenMatchesGolden(tester, 'error.light');
    });
  });

  group('widgets', () {
    testWidgets('cards are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DashboardScreen().withState<DashboardBloc, DashboardState>(
          MockDashboardBloc(),
          DashboardLoadSuccess([WalletMockData.card, WalletMockData.altCard]),
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
