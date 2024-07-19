import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/introduction/introduction_conditions_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';
import '../../util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('Conditions light', (tester) async {
      await tester.pumpDeviceBuilderWithAppWrapper(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IntroductionConditionsScreen(),
          ),
      );
      await tester.pumpAndSettle();
      await screenMatchesGolden(tester, 'conditions/light');
    });

    testGoldens('Conditions dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IntroductionConditionsScreen(),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await tester.pumpAndSettle();
      await screenMatchesGolden(tester, 'conditions/dark');
    });
  });

  group('widgets', () {
    testWidgets('conditions title and bullet points are shown', (WidgetTester tester) async {
      await tester.pumpWidgetWithAppWrapper(const IntroductionConditionsScreen());

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.introductionConditionsScreenHeadline), findsAtLeast(1));
      l10n.introductionConditionsScreenBulletPoints.split('\n').forEach((bulletPoint) {
        expect(find.text(bulletPoint), findsOneWidget);
      });
    });
  });
}
