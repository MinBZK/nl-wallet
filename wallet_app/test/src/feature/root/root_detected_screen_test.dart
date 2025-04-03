import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/root/root_detected_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('WalletPersonalizeNoDigidScreen Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RootDetectedScreen(),
      );
      await screenMatchesGolden('light');
    });

    testGoldens('WalletPersonalizeNoDigidScreen Dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RootDetectedScreen(),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('dark');
    });
  });

  group('widgets', () {
    testWidgets('description is shown in paragraphs', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const RootDetectedScreen());
      final l10n = await TestUtils.englishLocalizations;
      l10n.rootDetectedScreenDescription.split('\n\n').forEach(
        (paragraph) {
          expect(find.text(paragraph), findsOneWidget);
        },
      );
    });
  });
}
