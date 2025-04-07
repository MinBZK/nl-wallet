import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/forgot_pin/forgot_pin_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('forgot pin light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ForgotPinScreen(),
      );
      await screenMatchesGolden('light');
    });

    testGoldens('forgot pin dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ForgotPinScreen(),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('dark');
    });
  });

  group('widgets', () {
    testWidgets('clear wallet button can be found', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const ForgotPinScreen());
      final l10n = await TestUtils.englishLocalizations;
      final clearWalletButton = find.text(l10n.forgotPinScreenCta, findRichText: true);
      expect(clearWalletButton, findsOneWidget);
    });
  });
}
