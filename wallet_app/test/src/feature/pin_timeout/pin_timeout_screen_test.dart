import 'dart:ui';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/l10n/generated/app_localizations.dart';
import 'package:wallet/src/feature/pin_timeout/pin_timeout_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('PinTimeoutScreen light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            final expiryTime = DateTime.now().add(const Duration(seconds: 15, milliseconds: 500));
            return PinTimeoutScreen(expiryTime: expiryTime);
          },
        ),
      );
      await screenMatchesGolden('light');
    });

    testGoldens('PinTimeoutScreen light - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            final expiryTime = DateTime.now().add(const Duration(seconds: 15, milliseconds: 500));
            return PinTimeoutScreen(expiryTime: expiryTime);
          },
        ),
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('light.landscape');
    });

    testGoldens('PinTimeoutScreen dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            final expiryTime = DateTime.now().add(const Duration(seconds: 15, milliseconds: 500));
            return PinTimeoutScreen(expiryTime: expiryTime);
          },
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('dark');
    });
  });

  group('widgets', () {
    testWidgets('verify PinTimeoutScreen renders expected text', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        PinTimeoutScreen(
          expiryTime: DateTime.now().add(const Duration(seconds: 5)),
        ),
      );

      final AppLocalizations locale = await TestUtils.englishLocalizations;
      final headlineFinder = find.textContaining(locale.pinTimeoutScreenHeadline, findRichText: true);
      final ctaFinder = find.textContaining(locale.pinTimeoutScreenForgotPinCta, findRichText: true);

      expect(headlineFinder, findsNWidgets(2) /*In content and appbar*/);
      expect(ctaFinder, findsOneWidget);
    });
  });
}
