import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/service/event/app_event_coordinator.dart';
import 'package:wallet/src/data/service/navigation_service.dart';
import 'package:wallet/src/domain/usecase/update/observe_version_state_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/observe_wallet_locked_usecase.dart';
import 'package:wallet/src/feature/banner/cubit/banner_cubit.dart';
import 'package:wallet/src/feature/dashboard/bloc/dashboard_bloc.dart';
import 'package:wallet/src/feature/dashboard/dashboard_screen.dart';
import 'package:wallet/src/util/extension/localized_text_extension.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mock_data.dart';
import '../../mocks/wallet_mocks.dart';
import '../../test_util/golden_utils.dart';

class MockDashboardBloc extends MockBloc<DashboardEvent, DashboardState> implements DashboardBloc {}

void main() {
  group('goldens', () {
    testGoldens('ltc24 DashboardLoadSuccess light', (tester) async {
      await _pumpSuccessWithVersionState(tester, state: VersionStateOk());
      await screenMatchesGolden('success.light');
    });

    testGoldens('ltc24 DashboardLoadSuccess light - landscape', (tester) async {
      await _pumpSuccessWithVersionState(
        tester,
        state: VersionStateOk(),
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('success.light.landscape');
    });

    testGoldens('ltc24 DashboardLoadSuccess light - landscape - scrolled', (tester) async {
      await _pumpSuccessWithVersionState(
        tester,
        state: VersionStateOk(),
        surfaceSize: iphoneXSizeLandscape,
      );
      await tester.fling(find.byType(Scrollable).first, const Offset(0, -1000), 5000);
      await tester.pumpAndSettle();

      await screenMatchesGolden('success.light.landscape.scrolled');
    });

    testGoldens('ltc24 DashboardLoadSuccess dark', (tester) async {
      await _pumpSuccessWithVersionState(tester, state: VersionStateOk(), brightness: Brightness.dark);
      await screenMatchesGolden('success.dark');
    });

    testGoldens('ltc24 DashboardLoadInProgress light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DashboardScreen().withState<DashboardBloc, DashboardState>(
          MockDashboardBloc(),
          const DashboardLoadInProgress(),
        ),
        providers: [
          RepositoryProvider<ObserveWalletLockedUseCase>(create: (c) => MockObserveWalletLockedUseCase()),
          RepositoryProvider<AppEventCoordinator>(create: (c) => MockAppEventCoordinator()),
          RepositoryProvider<NavigationService>(create: (c) => MockNavigationService()),
        ],
      );
      await screenMatchesGolden('loading.light');
    });

    testGoldens('ltc24 DashboardLoadFailure light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DashboardScreen().withState<DashboardBloc, DashboardState>(
          MockDashboardBloc(),
          const DashboardLoadFailure(),
        ),
        providers: [
          RepositoryProvider<ObserveWalletLockedUseCase>(create: (c) => MockObserveWalletLockedUseCase()),
          RepositoryProvider<AppEventCoordinator>(create: (c) => MockAppEventCoordinator()),
          RepositoryProvider<NavigationService>(create: (c) => MockNavigationService()),
        ],
      );
      await screenMatchesGolden('error.light');
    });

    group('VersionState goldens', () {
      testGoldens('ltc24 DashboardLoadSuccess light - VersionStateNotify', (tester) async {
        await _pumpSuccessWithVersionState(tester, state: VersionStateNotify());
        await screenMatchesGolden('success.notify.light');
      });

      testGoldens('ltc24 DashboardLoadSuccess dark - VersionStateNotify', (tester) async {
        await _pumpSuccessWithVersionState(tester, state: VersionStateNotify(), brightness: Brightness.dark);
        await screenMatchesGolden('success.notify.dark');
      });

      testGoldens('ltc24 DashboardLoadSuccess light - VersionStateRecommend', (tester) async {
        await _pumpSuccessWithVersionState(tester, state: VersionStateRecommend());
        await screenMatchesGolden('success.recommend.light');
      });

      testGoldens('ltc24 DashboardLoadSuccess dark - VersionStateRecommend', (tester) async {
        await _pumpSuccessWithVersionState(tester, state: VersionStateRecommend(), brightness: Brightness.dark);
        await screenMatchesGolden('success.recommend.dark');
      });

      testGoldens('ltc24 DashboardLoadSuccess light - VersionStateWarn (10 days)', (tester) async {
        await _pumpSuccessWithVersionState(tester, state: VersionStateWarn(timeUntilBlocked: const Duration(days: 10)));
        await screenMatchesGolden('success.warn.10days.light');
      });

      testGoldens('ltc24 DashboardLoadSuccess dark - VersionStateWarn (10 days)', (tester) async {
        await _pumpSuccessWithVersionState(
          tester,
          state: VersionStateWarn(timeUntilBlocked: const Duration(days: 10)),
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('success.warn.10days.dark');
      });

      testGoldens('ltc24 DashboardLoadSuccess light - VersionStateWarn (10 hours)', (tester) async {
        await _pumpSuccessWithVersionState(
          tester,
          state: VersionStateWarn(timeUntilBlocked: const Duration(hours: 10)),
        );
        await screenMatchesGolden('success.warn.10hours.light');
      });

      testGoldens('ltc24 DashboardLoadSuccess dark - VersionStateWarn (10 hours)', (tester) async {
        await _pumpSuccessWithVersionState(
          tester,
          state: VersionStateWarn(timeUntilBlocked: const Duration(hours: 10)),
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('success.warn.10hours.dark');
      });

      testGoldens('ltc24 DashboardLoadSuccess light - VersionStateWarn (10 minutes)', (tester) async {
        await _pumpSuccessWithVersionState(
          tester,
          state: VersionStateWarn(timeUntilBlocked: const Duration(minutes: 10)),
        );
        await screenMatchesGolden('success.warn.10minutes.light');
      });

      testGoldens('ltc24 DashboardLoadSuccess light - VersionStateWarn (10 minutes)', (tester) async {
        await _pumpSuccessWithVersionState(
          tester,
          state: VersionStateWarn(timeUntilBlocked: const Duration(minutes: 10)),
          textScaleSize: 2,
        );
        await screenMatchesGolden('success.warn.10minutes.light.scaled_2x');
      });

      testGoldens('ltc24 DashboardLoadSuccess light - Cards', (tester) async {
        await _pumpSuccessWithVersionState(
          tester,
          state: VersionStateOk(),
          textScaleSize: 2,
        );
        final scrollableFinder = find.byType(Scrollable);
        await tester.scrollUntilVisible(
          find.text(WalletMockData.altCard.title.testValue),
          500,
          scrollable: scrollableFinder.first,
        );
        await screenMatchesGolden('success.ok.cards.light.scaled_2x');
      });

      testGoldens('ltc24 DashboardLoadSuccess dark - VersionStateWarn (10 minutes)', (tester) async {
        await _pumpSuccessWithVersionState(
          tester,
          state: VersionStateWarn(timeUntilBlocked: const Duration(minutes: 10)),
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('success.warn.10minutes.dark');
      });

      // Note: Not testing block as that is not rendered here, blocked state would simply lead to a non-functioning app.
    });
  });

  group('widgets', () {
    testWidgets('ltc24 cards are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DashboardScreen().withState<DashboardBloc, DashboardState>(
          MockDashboardBloc(),
          DashboardLoadSuccess(cards: [WalletMockData.card, WalletMockData.altCard], history: const []),
        ),
        providers: [
          RepositoryProvider<AppEventCoordinator>(create: (c) => MockAppEventCoordinator()),
          RepositoryProvider<NavigationService>(create: (c) => MockNavigationService()),
          RepositoryProvider<ObserveWalletLockedUseCase>(create: (c) => MockObserveWalletLockedUseCase()),
          RepositoryProvider<ObserveVersionStateUsecase>(create: (c) => MockObserveVersionStateUsecase()),
          RepositoryProvider<BannerCubit>(
            create: (c) => BannerCubit(
              MockObserveShowTourBannerUseCase(),
              MockObserveVersionStateUsecase(),
              MockObserveDashboardNotificationsUseCase(),
            ),
          ),
        ],
      );

      // Validate that the widget exists
      final cardTitleFinder = find.text(WalletMockData.card.title.testValue);
      final altCardTitleFinder = find.text(WalletMockData.altCard.title.testValue);
      expect(cardTitleFinder, findsOneWidget);
      expect(altCardTitleFinder, findsOneWidget);
    });
  });
}

/// Helper method that pumps the dashboard with the DashboardLoadSuccess state and the provided VersionState
Future<void> _pumpSuccessWithVersionState(
  WidgetTester tester, {
  required VersionState state,
  Brightness brightness = Brightness.light,
  double textScaleSize = 1,
  Size surfaceSize = iphoneXSize,
}) async {
  await tester.pumpWidgetWithAppWrapper(
    const DashboardScreen().withState<DashboardBloc, DashboardState>(
      MockDashboardBloc(),
      DashboardLoadSuccess(cards: [WalletMockData.card, WalletMockData.altCard], history: const []),
    ),
    brightness: brightness,
    textScaleSize: textScaleSize,
    surfaceSize: surfaceSize,
    providers: [
      RepositoryProvider<AppEventCoordinator>(create: (c) => MockAppEventCoordinator()),
      RepositoryProvider<ObserveWalletLockedUseCase>(create: (c) => MockObserveWalletLockedUseCase()),
      RepositoryProvider<NavigationService>(create: (c) => MockNavigationService()),
      RepositoryProvider<ObserveVersionStateUsecase>(
        create: (c) {
          final mockObserveVersionStateUsecase = MockObserveVersionStateUsecase();
          when(mockObserveVersionStateUsecase.invoke()).thenAnswer((_) => Stream.value(state));
          return mockObserveVersionStateUsecase;
        },
      ),
      RepositoryProvider<BannerCubit>(
        create: (c) {
          final versionStateUseCase = MockObserveVersionStateUsecase();
          when(versionStateUseCase.invoke()).thenAnswer((_) => Stream.value(state));
          final mockShowBannerUseCase = MockObserveShowTourBannerUseCase();
          when(mockShowBannerUseCase.invoke()).thenAnswer((_) => Stream.value(false));
          final mockObserveDashboardNotificationsUseCase = MockObserveDashboardNotificationsUseCase();
          when(mockObserveDashboardNotificationsUseCase.invoke()).thenAnswer((_) => Stream.value([]));
          return BannerCubit(
            mockShowBannerUseCase,
            versionStateUseCase,
            mockObserveDashboardNotificationsUseCase,
          );
        },
      ),
    ],
  );
}
