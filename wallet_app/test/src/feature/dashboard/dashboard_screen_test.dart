import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/data/service/navigation_service.dart';
import 'package:wallet/src/domain/usecase/update/observe_version_state_usecase.dart';
import 'package:wallet/src/feature/dashboard/bloc/dashboard_bloc.dart';
import 'package:wallet/src/feature/dashboard/dashboard_screen.dart';
import 'package:wallet/src/util/extension/localized_text_extension.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mock_data.dart';
import '../../mocks/wallet_mocks.dart';
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
              DashboardLoadSuccess(cards: [WalletMockData.card, WalletMockData.altCard], history: const []),
            ),
          ),
        wrapper: walletAppWrapper(
          providers: [
            RepositoryProvider<NavigationService>(create: (c) => MockNavigationService()),
            RepositoryProvider<ObserveVersionStateUsecase>(create: (c) => MockObserveVersionStateUsecase()),
          ],
        ),
      );
      await screenMatchesGolden(tester, 'success.light');
    });

    testGoldens('DashboardLoadSuccess dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DashboardScreen().withState<DashboardBloc, DashboardState>(
              MockDashboardBloc(),
              DashboardLoadSuccess(cards: [WalletMockData.card, WalletMockData.altCard], history: const []),
            ),
          ),
        wrapper: walletAppWrapper(
          brightness: Brightness.dark,
          providers: [
            RepositoryProvider<NavigationService>(create: (c) => MockNavigationService()),
            RepositoryProvider<ObserveVersionStateUsecase>(create: (c) => MockObserveVersionStateUsecase()),
          ],
        ),
      );
      await screenMatchesGolden(tester, 'success.dark');
    });

    testGoldens('DashboardLoadInProgress light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DashboardScreen().withState<DashboardBloc, DashboardState>(
          MockDashboardBloc(),
          const DashboardLoadInProgress(),
        ),
        providers: [
          RepositoryProvider<NavigationService>(create: (c) => MockNavigationService()),
        ],
      );
      await screenMatchesGolden(tester, 'loading.light');
    });

    testGoldens('DashboardLoadFailure light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DashboardScreen().withState<DashboardBloc, DashboardState>(
          MockDashboardBloc(),
          const DashboardLoadFailure(),
        ),
        providers: [
          RepositoryProvider<NavigationService>(create: (c) => MockNavigationService()),
        ],
      );
      await screenMatchesGolden(tester, 'error.light');
    });
  });

  group('widgets', () {
    testWidgets('cards are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DashboardScreen().withState<DashboardBloc, DashboardState>(
          MockDashboardBloc(),
          DashboardLoadSuccess(cards: [WalletMockData.card, WalletMockData.altCard], history: const []),
        ),
        providers: [
          RepositoryProvider<NavigationService>(create: (c) => MockNavigationService()),
          RepositoryProvider<ObserveVersionStateUsecase>(create: (c) => MockObserveVersionStateUsecase()),
        ],
      );

      // Validate that the widget exists
      final cardTitleFinder = find.text(WalletMockData.card.front.title.testValue);
      final altCardTitleFinder = find.text(WalletMockData.altCard.front.title.testValue);
      expect(cardTitleFinder, findsOneWidget);
      expect(altCardTitleFinder, findsOneWidget);
    });
  });
}
