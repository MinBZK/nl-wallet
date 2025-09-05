import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/usecase/biometrics/get_available_biometrics_usecase.dart';
import 'package:wallet/src/feature/splash/bloc/splash_bloc.dart';
import 'package:wallet/src/feature/splash/splash_screen.dart';
import 'package:wallet/src/navigation/wallet_routes.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.dart';
import '../../test_util/golden_utils.dart';

class MockSplashBloc extends MockBloc<SplashEvent, SplashState> implements SplashBloc {}

void main() {
  group('goldens', () {
    testGoldens('SplashScreen initial light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SplashScreen().withState<SplashBloc, SplashState>(
          MockSplashBloc(),
          SplashInitial(),
        ),
        providers: [
          RepositoryProvider<GetAvailableBiometricsUseCase>(create: (context) => MockGetAvailableBiometricsUseCase()),
        ],
      );
      await screenMatchesGolden('initial.light');
    });

    testGoldens('SplashScreen initial dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SplashScreen().withState<SplashBloc, SplashState>(
          MockSplashBloc(),
          SplashInitial(),
        ),
        brightness: Brightness.dark,
        providers: [
          RepositoryProvider<GetAvailableBiometricsUseCase>(create: (context) => MockGetAvailableBiometricsUseCase()),
        ],
      );
      await screenMatchesGolden('initial.dark');
    });
  });

  group('widgets', () {
    testWidgets('when NOT registered navigate to introduction', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SplashScreen().withState<SplashBloc, SplashState>(
          MockSplashBloc(),
          const SplashLoaded(isRegistered: false, hasPid: false),
        ),
        providers: [
          RepositoryProvider<GetAvailableBiometricsUseCase>(create: (context) => MockGetAvailableBiometricsUseCase()),
        ],
      );
      await tester.pumpAndSettle();
      expect(find.text(WalletRoutes.introductionRoute), findsOneWidget);
    });

    testWidgets('when registered and pid is NOT available navigate to personalization', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SplashScreen().withState<SplashBloc, SplashState>(
          MockSplashBloc(),
          const SplashLoaded(isRegistered: true, hasPid: false),
        ),
        providers: [
          RepositoryProvider<GetAvailableBiometricsUseCase>(create: (context) => MockGetAvailableBiometricsUseCase()),
        ],
      );
      await tester.pumpAndSettle();
      expect(find.text(WalletRoutes.walletPersonalizeRoute), findsOneWidget);
    });

    testWidgets('when registered and pid is available navigate to home', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SplashScreen().withState<SplashBloc, SplashState>(
          MockSplashBloc(),
          const SplashLoaded(isRegistered: true, hasPid: true),
        ),
        providers: [
          RepositoryProvider<GetAvailableBiometricsUseCase>(create: (context) => MockGetAvailableBiometricsUseCase()),
        ],
      );
      await tester.pumpAndSettle();
      expect(find.text(WalletRoutes.dashboardRoute), findsOneWidget);
    });
  });
}
