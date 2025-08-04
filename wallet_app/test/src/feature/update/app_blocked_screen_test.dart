import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/update/app_blocked_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('Light Test', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AppBlockedScreen(),
      );
      await tester.pumpAndSettle();

      await screenMatchesGolden('blocked.light');
    });

    testGoldens('Dark Test', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AppBlockedScreen(),
        brightness: Brightness.dark,
      );
      await tester.pumpAndSettle();

      await screenMatchesGolden('blocked.dark');
    });
  });

  group('widgets', () {
    testWidgets('Title is shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AppBlockedScreen(),
      );

      final l10n = await TestUtils.englishLocalizations;

      final titleFinder = find.textContaining(l10n.appBlockedScreenTitle, findRichText: true);
      expect(titleFinder, findsOneWidget);
    });
  });
}
