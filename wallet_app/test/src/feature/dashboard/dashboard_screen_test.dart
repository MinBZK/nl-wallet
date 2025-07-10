import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/service/navigation_service.dart';
import 'package:wallet/src/domain/usecase/update/observe_version_state_usecase.dart';
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
    testGoldens('DashboardLoadSuccess light', (tester) async {
      await _pumpSuccessWithVersionState(tester, state: VersionStateOk());
      await screenMatchesGolden('success.light');
    });

    testGoldens('DashboardLoadSuccess dark', (tester) async {
      await _pumpSuccessWithVersionState(tester, state: VersionStateOk(), brightness: Brightness.dark);
      await screenMatchesGolden('success.dark');
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
      await screenMatchesGolden('loading.light');
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
      await screenMatchesGolden('error.light');
    });

    group('VersionState goldens', () {
      testGoldens('DashboardLoadSuccess light - VersionStateNotify', (tester) async {
        await _pumpSuccessWithVersionState(tester, state: VersionStateNotify());
        await screenMatchesGolden('success.notify.light');
      });

      testGoldens('DashboardLoadSuccess dark - VersionStateNotify', (tester) async {
        await _pumpSuccessWithVersionState(tester, state: VersionStateNotify(), brightness: Brightness.dark);
        await screenMatchesGolden('success.notify.dark');
      });

      testGoldens('DashboardLoadSuccess light - VersionStateRecommend', (tester) async {
        await _pumpSuccessWithVersionState(tester, state: VersionStateRecommend());
        await screenMatchesGolden('success.recommend.light');
      });

      testGoldens('DashboardLoadSuccess dark - VersionStateRecommend', (tester) async {
        await _pumpSuccessWithVersionState(tester, state: VersionStateRecommend(), brightness: Brightness.dark);
        await screenMatchesGolden('success.recommend.dark');
      });

      testGoldens('DashboardLoadSuccess light - VersionStateWarn (10 days)', (tester) async {
        await _pumpSuccessWithVersionState(tester, state: VersionStateWarn(timeUntilBlocked: const Duration(days: 10)));
        await screenMatchesGolden('success.warn.10days.light');
      });

      testGoldens('DashboardLoadSuccess dark - VersionStateWarn (10 days)', (tester) async {
        await _pumpSuccessWithVersionState(
          tester,
          state: VersionStateWarn(timeUntilBlocked: const Duration(days: 10)),
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('success.warn.10days.dark');
      });

      testGoldens('DashboardLoadSuccess light - VersionStateWarn (10 hours)', (tester) async {
        await _pumpSuccessWithVersionState(
          tester,
          state: VersionStateWarn(timeUntilBlocked: const Duration(hours: 10)),
        );
        await screenMatchesGolden('success.warn.10hours.light');
      });

      testGoldens('DashboardLoadSuccess dark - VersionStateWarn (10 hours)', (tester) async {
        await _pumpSuccessWithVersionState(
          tester,
          state: VersionStateWarn(timeUntilBlocked: const Duration(hours: 10)),
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('success.warn.10hours.dark');
      });

      testGoldens('DashboardLoadSuccess light - VersionStateWarn (10 minutes)', (tester) async {
        await _pumpSuccessWithVersionState(
          tester,
          state: VersionStateWarn(timeUntilBlocked: const Duration(minutes: 10)),
        );
        await screenMatchesGolden('success.warn.10minutes.light');
      });

      testGoldens('DashboardLoadSuccess light - VersionStateWarn (10 minutes)', (tester) async {
        await _pumpSuccessWithVersionState(
          tester,
          state: VersionStateWarn(timeUntilBlocked: const Duration(minutes: 10)),
          textScaleSize: 2,
        );
        await screenMatchesGolden('success.warn.10minutes.light.scaled_2x');
      });

      testGoldens('DashboardLoadSuccess light - Cards', (tester) async {
        await _pumpSuccessWithVersionState(
          tester,
          state: VersionStateOk(),
          textScaleSize: 2,
        );
        final scrollableFinder = find.byType(Scrollable);
        await tester.scrollUntilVisible(
          find.text(WalletMockData.altCard.title.testValue),
          500,
          scrollable: scrollableFinder,
        );
        await screenMatchesGolden('success.ok.cards.light.scaled_2x');
      });

      testGoldens('DashboardLoadSuccess dark - VersionStateWarn (10 minutes)', (tester) async {
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
    testWidgets('cards are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DashboardScreen().withState<DashboardBloc, DashboardState>(
          MockDashboardBloc(),
          DashboardLoadSuccess(cards: [WalletMockData.card, WalletMockData.altCard], history: const []),
        ),
        providers: [
          RepositoryProvider<NavigationService>(create: (c) => MockNavigationService()),
          RepositoryProvider<ObserveVersionStateUsecase>(create: (c) => MockObserveVersionStateUsecase()),
          RepositoryProvider<BannerCubit>(
            create: (c) => BannerCubit(
              MockObserveShowTourBannerUseCase(),
              MockObserveVersionStateUsecase(),
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
}) async {
  await tester.pumpWidgetWithAppWrapper(
    const DashboardScreen().withState<DashboardBloc, DashboardState>(
      MockDashboardBloc(),
      DashboardLoadSuccess(cards: [WalletMockData.card, WalletMockData.altCard], history: const []),
    ),
    brightness: brightness,
    textScaleSize: textScaleSize,
    providers: [
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
          return BannerCubit(mockShowBannerUseCase, versionStateUseCase);
        },
      ),
    ],
  );
}
