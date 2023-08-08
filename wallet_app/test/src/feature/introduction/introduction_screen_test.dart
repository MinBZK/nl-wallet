import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/introduction/introduction_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';

/// Note: The page indicator placement misbehaves when rendering multiple instances of the [IntroductionScreen]
/// in the same golden. To verify it's normal placement the [page_1.stepper.light] test is added.
void main() {
  group('goldens', () {
    testGoldens('Page 1 light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IntroductionScreen(),
            name: 'page_1',
          ),
        wrapper: walletAppWrapper(),
      );
      await tester.pumpAndSettle();
      await screenMatchesGolden(tester, 'page_1.light');
    });

    testGoldens('Page 2 light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IntroductionScreen(),
            name: 'page_2',
            onCreate: (scenarioWidgetKey) async {
              await _skipPage(scenarioWidgetKey, tester);
            },
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'page_2.light');
    });
    testGoldens('Page 3 light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IntroductionScreen(),
            name: 'page_3',
            onCreate: (scenarioWidgetKey) async {
              await _skipPage(scenarioWidgetKey, tester);
              await _skipPage(scenarioWidgetKey, tester);
            },
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'page_3.light');
    });
    testGoldens('Page 4 light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IntroductionScreen(),
            name: 'page_4',
            onCreate: (scenarioWidgetKey) async {
              await _skipPage(scenarioWidgetKey, tester);
              await _skipPage(scenarioWidgetKey, tester);
              await _skipPage(scenarioWidgetKey, tester);
            },
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'page_4.light');
    });

    testGoldens('Page 1 dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IntroductionScreen(),
            name: 'page_1',
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'page_1.dark');
    });

    testGoldens('Page 1 individual to render portrait and thus show stepper correctly', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const IntroductionScreen());
      await screenMatchesGolden(tester, 'page_1.stepper.light');
    });
  });
}

Future<void> _skipPage(Key scenarioWidgetKey, WidgetTester tester) async {
  final finder = find.descendant(
    of: find.byKey(scenarioWidgetKey),
    matching: find.byKey(const Key('introductionNextPageCta')),
  );
  expect(finder, findsOneWidget);

  await tester.tap(finder);
  await tester.pumpAndSettle();
}
