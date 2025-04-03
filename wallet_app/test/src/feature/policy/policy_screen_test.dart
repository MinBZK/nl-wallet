import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/policy/policy_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mock_data.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('Light Test', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        PolicyScreen(
          relyingParty: WalletMockData.organization,
          policy: WalletMockData.policy,
          onReportIssuePressed: () {},
        ),
      );
      await screenMatchesGolden('light');
    });

    testGoldens('Dark Test', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        PolicyScreen(
          relyingParty: WalletMockData.organization,
          policy: WalletMockData.policy,
          onReportIssuePressed: () {},
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('dark');
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
