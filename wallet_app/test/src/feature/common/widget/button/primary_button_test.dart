import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/button/primary_button.dart';

import '../../../../../wallet_app_test_widget.dart';

void main() {
  group('widgets', () {
    testWidgets('button text is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const PrimaryButton(
          text: Text('Button'),
        ),
      );

      final textFinder = find.text('Button');
      expect(textFinder, findsOneWidget);
    });

    testWidgets('default icon is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const PrimaryButton(
          text: Text('Button'),
        ),
      );

      final iconFinder = find.byIcon(Icons.arrow_forward_outlined);
      expect(iconFinder, findsOneWidget);
    });

    testWidgets('default icon is invisible when icon is set to null', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const PrimaryButton(
          text: Text('Button'),
          icon: null,
        ),
      );

      final iconFinder = find.byIcon(Icons.arrow_forward_outlined);
      expect(iconFinder, findsNothing);
    });

    testWidgets('custom icon is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const PrimaryButton(
          text: Text('Button'),
          icon: FlutterLogo(),
        ),
      );

      final iconFinder = find.byType(FlutterLogo);
      expect(iconFinder, findsOneWidget);
    });

    testWidgets('onPressed callback is triggered when clicked', (tester) async {
      bool isPressed = false;
      await tester.pumpWidgetWithAppWrapper(
        PrimaryButton(
          text: const Text('Button'),
          onPressed: () => isPressed = true,
        ),
      );

      final textFinder = find.text('Button');
      await tester.tap(textFinder);
      expect(isPressed, isTrue, reason: 'button callback not triggered');
    });
  });
}
