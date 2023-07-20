import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/policy/policy_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/mock_data.dart';
import '../../util/device_utils.dart';

void main() {
  group('goldens', () {
    DeviceBuilder deviceBuilder(WidgetTester tester) {
      return DeviceUtils.deviceBuilderWithPrimaryScrollController
        ..addScenario(
          widget: PolicyScreen(
            policy: WalletMockData.policy,
            onReportIssuePressed: () {},
          ),
        );
    }

    testGoldens('Light Test', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'light');
    });

    testGoldens('Dark Test', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'dark');
    });
  });

  group('widgets', () {
    testWidgets('onReportIssuePressed is triggered when button is tapped', (tester) async {
      bool isCalled = false;
      await tester.pumpWidgetWithAppWrapper(
        PolicyScreen(
          policy: WalletMockData.policy,
          onReportIssuePressed: () => isCalled = true,
        ),
        surfaceSize: const Size(350, 1000), //Extra high so button is immediately visible
      );

      final issueButtonFinder = find.text('Report an issue');
      expect(issueButtonFinder, findsOneWidget);
      await tester.tap(issueButtonFinder);
      expect(isCalled, isTrue);
    });
  });
}
