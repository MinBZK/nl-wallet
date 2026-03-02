import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/update/app_blocked_by_update_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('ltc43 Light Test', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AppBlockedByUpdateScreen(),
      );
      await tester.pumpAndSettle();

      await screenMatchesGolden('blocked.light');
    });

    testGoldens('ltc43 Dark Test', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AppBlockedByUpdateScreen(),
        brightness: Brightness.dark,
      );
      await tester.pumpAndSettle();

      await screenMatchesGolden('blocked.dark');
    });
  });

  group('widgets', () {
    testWidgets('ltc43 Title is shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AppBlockedByUpdateScreen(),
      );

      final l10n = await TestUtils.englishLocalizations;

      final titleFinder = find.textContaining(l10n.appBlockedByUpdateScreenTitle, findRichText: true);
      expect(titleFinder, findsOneWidget);
    });
  });
}
