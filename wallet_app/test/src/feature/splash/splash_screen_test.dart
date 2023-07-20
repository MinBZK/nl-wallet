import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/splash/bloc/splash_bloc.dart';
import 'package:wallet/src/feature/splash/splash_screen.dart';
import 'package:wallet/src/navigation/wallet_routes.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';

class MockSplashBloc extends MockBloc<SplashEvent, SplashState> implements SplashBloc {}

void main() {
  final deviceBuilder = DeviceUtils.deviceBuilder
    ..addScenario(
      widget: const SplashScreen().withState<SplashBloc, SplashState>(
        MockSplashBloc(),
        SplashInitial(),
      ),
    );

  group('goldens', () {
    testGoldens('SplashScreeon initial light', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder,
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'initial.light');
    });

    testGoldens('SplashScreen initial dark', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder,
        wrapper: walletAppWrapper(
          brightness: Brightness.dark,
        ),
      );
      await screenMatchesGolden(tester, 'initial.dark');
    });
  });

  group('widgets', () {
    testWidgets('when NOT registered navigate to introduction', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SplashScreen().withState<SplashBloc, SplashState>(
          MockSplashBloc(),
          const SplashLoaded(isRegistered: false, hasPid: false),
        ),
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
      );
      await tester.pumpAndSettle();
      expect(find.text(WalletRoutes.homeRoute), findsOneWidget);
    });
  });
}
