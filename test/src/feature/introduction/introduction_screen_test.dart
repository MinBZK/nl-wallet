import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/introduction/introduction_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';

void main() {
  group('Golden Tests', () {
    testGoldens(
      'Accessibility Test',
      (tester) async {
        final deviceBuilder = DeviceUtils.accessibilityDeviceBuilder;
        deviceBuilder.addScenario(
          widget: const IntroductionScreen(),
          name: 'page_1',
        );
        deviceBuilder.addScenario(
          widget: const IntroductionScreen(),
          name: 'page_2',
          onCreate: (scenarioWidgetKey) async {
            await _skipPage(scenarioWidgetKey, tester);
          },
        );
        deviceBuilder.addScenario(
          widget: const IntroductionScreen(),
          name: 'page_3',
          onCreate: (scenarioWidgetKey) async {
            await _skipPage(scenarioWidgetKey, tester);
            await _skipPage(scenarioWidgetKey, tester);
          },
        );
        deviceBuilder.addScenario(
          widget: const IntroductionScreen(),
          name: 'page_4',
          onCreate: (scenarioWidgetKey) async {
            await _skipPage(scenarioWidgetKey, tester);
            await _skipPage(scenarioWidgetKey, tester);
            await _skipPage(scenarioWidgetKey, tester);
          },
        );

        await tester.pumpDeviceBuilder(deviceBuilder, wrapper: walletAppWrapper());
        await screenMatchesGolden(tester, 'accessibility_scaling');
      },
    );
  });
}

Future<void> _skipPage(Key scenarioWidgetKey, WidgetTester tester) async {
  final finder = find.descendant(
    of: find.byKey(scenarioWidgetKey),
    matching: find.byIcon(Icons.arrow_forward),
  );
  expect(finder, findsOneWidget);

  await tester.tap(finder);
  await tester.pumpAndSettle();
}
