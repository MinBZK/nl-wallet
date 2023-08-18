import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/introduction/introduction_privacy_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('IntroductionPrivacyScreen light', (tester) async {
      await tester.pumpDeviceBuilderWithAppWrapper(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IntroductionPrivacyScreen(),
          ),
      );
      await tester.pumpAndSettle();
      await screenMatchesGolden(tester, 'privacy/light');
    });

    testGoldens('IntroductionPrivacyScreen dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IntroductionPrivacyScreen(),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await tester.pumpAndSettle();
      await screenMatchesGolden(tester, 'privacy/dark');
    });
  });
}
