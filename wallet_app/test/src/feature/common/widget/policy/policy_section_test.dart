import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/widget/policy/policy_section.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mock_data.dart';

void main() {
  const kGoldenSize = Size(300, 192);

  group('goldens', () {
    testGoldens(
      'light policy section',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          PolicySection(relyingParty: WalletMockData.organization, policy: WalletMockData.policy),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'policy_section/light');
      },
    );
    testGoldens(
      'dark policy section',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          PolicySection(relyingParty: WalletMockData.organization, policy: WalletMockData.policy),
          brightness: Brightness.dark,
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'policy_section/dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('widget is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        PolicySection(relyingParty: WalletMockData.organization, policy: WalletMockData.policy),
      );

      // Validate that the widget exists
      final widgetFinder = find.text('All terms');
      expect(widgetFinder, findsOneWidget);
    });
  });
}
