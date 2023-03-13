import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/splash/bloc/splash_bloc.dart';
import 'package:wallet/src/feature/splash/splash_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';

class MockSplashBloc extends MockBloc<SplashEvent, SplashState> implements SplashBloc {}

void main() {
  final SplashBloc splashBloc = MockSplashBloc();

  setUp(() {
    whenListen(
      splashBloc,
      Stream.fromIterable([SplashInitial()]),
      initialState: SplashInitial(),
    );
  });

  group('Golden Tests', () {
    testGoldens(
      'Accessibility Test',
      (tester) async {
        final deviceBuilder = DeviceUtils.accessibilityDeviceBuilder;
        deviceBuilder.addScenario(widget: const SplashScreen());

        await tester.pumpDeviceBuilder(
          deviceBuilder,
          wrapper: walletAppWrapper(
            providers: [BlocProvider<SplashBloc>(create: (c) => splashBloc)],
          ),
        );
        await screenMatchesGolden(tester, 'accessibility_scaling');
      },
    );
  });
}
