import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/policy/policy_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mock_data.dart';
import '../../util/device_utils.dart';
import '../../util/test_utils.dart';

void main() {
  group('goldens', () {
    DeviceBuilder deviceBuilder(WidgetTester tester) {
      return DeviceUtils.deviceBuilderWithPrimaryScrollController
        ..addScenario(
          widget: PolicyScreen(
            relyingParty: WalletMockData.organization,
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
          relyingParty: WalletMockData.organization,
          policy: WalletMockData.policy,
          onReportIssuePressed: () => isCalled = true,
        ),
        surfaceSize: const Size(350, 1200), //Extra high so button is immediately visible
      );

      final l10n = await TestUtils.englishLocalizations;
      final issueButtonFinder = find.text(l10n.policyScreenReportIssueCta);
      expect(issueButtonFinder, findsOneWidget);
      await tester.tap(issueButtonFinder);
      expect(isCalled, isTrue);
    });
  });
}
