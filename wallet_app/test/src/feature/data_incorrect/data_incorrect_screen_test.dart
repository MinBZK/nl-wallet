import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/data_incorrect/data_incorrect_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('DataIncorrectScreen light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DataIncorrectScreen(),
      );
      await screenMatchesGolden('light');
    });

    testGoldens('DataIncorrectScreen dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DataIncorrectScreen(),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('dark');
    });
  });

  group('widgets', () {
    testWidgets('data screen renders as expected', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const DataIncorrectScreen());
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.dataIncorrectScreenHeaderTitle), findsAtLeast(1));
      expect(find.text(l10n.dataIncorrectScreenHeaderDescription), findsOneWidget);
      // Accept and decline CTAs are visible
      expect(find.text(l10n.dataIncorrectScreenDeclineCta), findsNWidgets(2 /* title & cta */));
      expect(find.text(l10n.dataIncorrectScreenApproveCta), findsOneWidget);
    });
  });
}
