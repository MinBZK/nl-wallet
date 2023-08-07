import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/introduction/introduction_expectations_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('Expectations light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IntroductionExpectationsScreen(),
          ),
        wrapper: walletAppWrapper(),
      );
      await tester.pumpAndSettle();
      await screenMatchesGolden(tester, 'expectations/light');
    });

    testGoldens('Expectations dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IntroductionExpectationsScreen(),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await tester.pumpAndSettle();
      await screenMatchesGolden(tester, 'expectations/dark');
    });
  });
}
