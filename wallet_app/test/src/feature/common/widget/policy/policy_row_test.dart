import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/widget/policy/policy_row.dart';

import '../../../../../wallet_app_test_widget.dart';

void main() {
  const kGoldenSize = Size(150, 38);

  group('goldens', () {
    testGoldens(
      'light policy row',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const PolicyRow(title: 'Title', icon: Icons.security_outlined),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'policy_row/light');
      },
    );
    testGoldens(
      'dark policy row',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const PolicyRow(title: 'Title', icon: Icons.security_outlined),
          brightness: Brightness.dark,
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'policy_row/dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('widget is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const PolicyRow(title: 'Title', icon: Icons.security_outlined),
      );

      // Validate that the widget exists
      final widgetFinder = find.text('Title');
      expect(widgetFinder, findsOneWidget);
    });
  });
}
