import 'dart:ui';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/l10n/generated/app_localizations.dart';
import 'package:wallet/src/feature/pin_timeout/argument/pin_timeout_screen_argument.dart';
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

    testWidgets('verify onExpire is called', (tester) async {
      bool isCalled = false;

      // Provide an expiryTime in the past, causing the callback to trigger asap
      await tester.pumpWidgetWithAppWrapper(
        PinTimeoutScreen(
          expiryTime: DateTime.now().subtract(const Duration(seconds: 1)),
          onExpire: () => isCalled = true,
        ),
      );

      // Advance the timer so that expiry will be checked
      await tester.pump(const Duration(seconds: 1));
      await tester.pumpAndSettle();

      // Verify callback is executed
      expect(isCalled, isTrue);
    });
  });

  group('PinTimeoutScreenArgument', () {
    test('getArgument is (de)serializable through the getArgument method', () {
      final expiry = DateTime(2020);
      final input = PinTimeoutScreenArgument(expiryTime: expiry);
      final result = PinTimeoutScreen.getArgument(RouteSettings(arguments: input.toMap()));
      expect(result, input);
    });

    test('getArgument throws for invalid input', () {
      expect(() => PinTimeoutScreen.getArgument(const RouteSettings(arguments: 1)), throwsA(isA<UnsupportedError>()));
    });
  });
}
