import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/introduction/introduction_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';
import '../../util/test_utils.dart';

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

  group('widgets', () {
    testWidgets('page 1 title and description are shown', (WidgetTester tester) async {
      await tester.pumpWidgetWithAppWrapper(const IntroductionScreen());

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.introductionPage1Title), findsAtLeast(1));
      expect(find.text(l10n.introductionPage1Description), findsOneWidget);
    });

    testWidgets('play/pause button initially shows pause icon', (WidgetTester tester) async {
      await tester.pumpWidgetWithAppWrapper(const IntroductionScreen());

      expect(find.byIcon(Icons.pause_outlined), findsOneWidget);
    });

    testWidgets('play/pause button shows play icon after tapping it', (WidgetTester tester) async {
      await tester.pumpWidgetWithAppWrapper(const IntroductionScreen());

      await tester.tap(find.byIcon(Icons.pause_outlined));
      await tester.pumpAndSettle();

      expect(find.byIcon(Icons.play_arrow_rounded), findsOneWidget);
    });

    testWidgets('page 2 title and description are shown', (WidgetTester tester) async {
      const Key key = Key('introduction');
      await tester.pumpWidgetWithAppWrapper(const IntroductionScreen(key: key));

      await _skipPage(key, tester);

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.introductionPage2Title), findsAtLeast(1));
      expect(find.text(l10n.introductionPage2Description), findsOneWidget);
    });

    testWidgets('page 3 title and description are shown', (WidgetTester tester) async {
      const Key key = Key('introduction');
      await tester.pumpWidgetWithAppWrapper(const IntroductionScreen(key: key));

      await _skipPage(key, tester);
      await _skipPage(key, tester);

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.introductionPage3Title), findsAtLeast(1));
      expect(find.text(l10n.introductionPage3Description), findsOneWidget);
    });
  });
}

Future<void> _skipPage(Key scenarioWidgetKey, WidgetTester tester) async {
  final l10n = await TestUtils.englishLocalizations;
  final finder = find.descendant(
    of: find.byKey(scenarioWidgetKey),
    matching: find.text(l10n.introductionNextPageCta),
  );
  expect(finder, findsOneWidget);

  await tester.tap(finder);
  await tester.pumpAndSettle();
}
