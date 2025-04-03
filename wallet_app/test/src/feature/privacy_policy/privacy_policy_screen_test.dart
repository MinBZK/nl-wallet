import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/privacy_policy/privacy_policy_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';

void main() {
  /// This is required to read from the rootBundle (containing policy.md) multiple times
  tearDown(rootBundle.clear);

  group('goldens', () {
    testGoldens('Light Full Text Test', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const PrivacyPolicyScreen(),
        surfaceSize: const Size(1500, 3500),
      );
      await tester.pumpAndSettle();

      await screenMatchesGolden('light.full');
    });

    testGoldens('Light Test', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const PrivacyPolicyScreen(),
      );
      await tester.pumpAndSettle();

      await screenMatchesGolden('light');
    });

    testGoldens('Dark Test', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const PrivacyPolicyScreen(),
        brightness: Brightness.dark,
      );
      await tester.pumpAndSettle();

      await screenMatchesGolden('dark');
    });
  });

  group('widgets', () {
    testWidgets('Policy was last updated 04/12/2024', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const PrivacyPolicyScreen(),
      );
      await tester.pumpAndSettle();

      final dateFinder = find.textContaining('4 december 2024');
      expect(dateFinder, findsOneWidget);
    });
  });
}
