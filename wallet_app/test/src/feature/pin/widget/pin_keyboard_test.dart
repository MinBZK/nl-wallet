// This is an example Flutter widget test.
//
// To perform an interaction with a widget in your test, use the WidgetTester
// utility in the flutter_test package. For example, you can send tap and scroll
// gestures. You can also use WidgetTester to find child widgets in the widget
// tree, read text, and verify that the values of widget properties are correct.
//
// Visit https://flutter.dev/docs/cookbook/testing/widget/introduction for
// more information about Widget testing.

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/pin/widget/pin_keyboard.dart';

import '../../../../wallet_app_test_widget.dart';

void main() {
  group('PinKeyboard', () {
    testWidgets('should display all numeric keys', (WidgetTester tester) async {
      await tester.pumpWidgetWithAppWrapper(const PinKeyboard());

      // Verify all pin options [1..9] are displayed
      for (int i = 0; i < 10; i++) {
        expect(find.text(i.toString(), findRichText: true), findsOneWidget);
      }
    });

    testWidgets('should display a backspace key', (WidgetTester tester) async {
      await tester.pumpWidgetWithAppWrapper(const PinKeyboard());

      expect(find.byIcon(Icons.keyboard_backspace_rounded), findsOneWidget);
    });

    testWidgets('should meet text contrast guidelines', (WidgetTester tester) async {
      final SemanticsHandle handle = tester.ensureSemantics();
      await tester.pumpWidgetWithAppWrapper(const PinKeyboard());
      await expectLater(tester, meetsGuideline(textContrastGuideline));
      handle.dispose();
    });

    testWidgets('should trigger `onKeyPressed` callback when a key is pressed', (WidgetTester tester) async {
      int lastPressedKey = -1;
      final pinKeyboard = PinKeyboard(
        onKeyPressed: (key) => lastPressedKey = key,
      );

      await tester.pumpWidgetWithAppWrapper(pinKeyboard);
      for (int i = 0; i < 10; i++) {
        final widgetFinder = find.text(i.toString(), findRichText: true);
        await tester.tap(widgetFinder);
        expect(i, lastPressedKey);
      }
    });

    testWidgets('should trigger `onBackspacePressed` callback when the backspace key is pressed',
        (WidgetTester tester) async {
      bool onBackspaceWasPressed = false;
      final pinKeyboard = PinKeyboard(
        onBackspacePressed: () => onBackspaceWasPressed = true,
      );

      await tester.pumpWidgetWithAppWrapper(pinKeyboard);
      final widgetFinder = find.byIcon(Icons.keyboard_backspace_rounded);
      await tester.tap(widgetFinder);

      expect(onBackspaceWasPressed, isTrue);
    });
  });
}
