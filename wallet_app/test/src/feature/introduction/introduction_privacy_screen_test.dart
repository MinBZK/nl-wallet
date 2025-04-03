import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/introduction/introduction_privacy_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('IntroductionPrivacyScreen light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IntroductionPrivacyScreen(),
      );
      await tester.pumpAndSettle();
      await screenMatchesGolden('privacy/light');
    });

    testGoldens('IntroductionPrivacyScreen dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IntroductionPrivacyScreen(),
        brightness: Brightness.dark,
      );
      await tester.pumpAndSettle();
      await screenMatchesGolden('privacy/dark');
    });
  });

  group('widgets', () {
    testWidgets('privacy title and bullet points are shown', (WidgetTester tester) async {
      await tester.pumpWidgetWithAppWrapper(const IntroductionPrivacyScreen());

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.introductionPrivacyScreenHeadline), findsAtLeast(1));
      l10n.introductionPrivacyScreenBulletPoints.split('\n').forEach((bulletPoint) {
        expect(find.text(bulletPoint), findsOneWidget);
      });
    });
  });
}
