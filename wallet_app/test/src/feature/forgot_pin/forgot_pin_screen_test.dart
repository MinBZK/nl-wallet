import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/forgot_pin/forgot_pin_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';
import '../../util/test_utils.dart';

void main() {
  DeviceBuilder deviceBuilder(WidgetTester tester) {
    return DeviceUtils.deviceBuilderWithPrimaryScrollController
      ..addScenario(
        widget: const ForgotPinScreen(),
      );
  }

  group('goldens', () {
    testGoldens('forgot pin light', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'light');
    });

    testGoldens('forgot pin dark', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'dark');
    });
  });

  group('widgets', () {
    testWidgets('clear wallet button can be found', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const ForgotPinScreen());
      final l10n = await TestUtils.englishLocalizations;
      final clearWalletButton = find.text(l10n.forgotPinScreenCta, findRichText: true);
      expect(clearWalletButton, findsOneWidget);
    });
  });
}
