import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/l10n/generated/app_localizations.dart';
import 'package:wallet/src/feature/pin_blocked/pin_blocked_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('ltc46 PinBlockedScreen light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const PinBlockedScreen(),
      );
      await screenMatchesGolden('light');
    });

    testGoldens('ltc46 PinBlockedScreen dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const PinBlockedScreen(),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('dark');
    });
  });

  group('widgets', () {
    testWidgets('ltc46 verify PinBlockedScreen renders expected text', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const PinBlockedScreen());

      final AppLocalizations l10n = await TestUtils.englishLocalizations;
      final headlineFinder = find.textContaining(l10n.pinBlockedScreenHeadline, findRichText: true);
      final descriptionFinder = find.textContaining(l10n.pinBlockedScreenDescription, findRichText: true);
      final ctaFinder = find.textContaining(l10n.pinBlockedScreenResetWalletCta, findRichText: true);

      expect(headlineFinder, findsOneWidget);
      expect(descriptionFinder, findsOneWidget);
      expect(ctaFinder, findsOneWidget);
    });
  });
}
