import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/utility/max_brightness.dart';

import '../../../../../wallet_app_test_widget.dart';

void main() {
  group('widgets', () {
    testWidgets('max brightness is set on init', (tester) async {
      double? selectedBrightness;
      // Mock the method channel to see if the widget actually sets the brightness
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
          const MethodChannel('github.com/aaassseee/screen_brightness'), (MethodCall methodCall) async {
        if (methodCall.method == 'setApplicationScreenBrightness') {
          selectedBrightness = methodCall.arguments['brightness'] as double;
        }
        return null;
      });

      await tester.pumpWidgetWithAppWrapper(const MaxBrightness(child: SizedBox.shrink()));

      // Check if the widget indeed requested setting the brightness to 1.0
      expect(selectedBrightness, 1.0);
    });

    testWidgets('child is rendered', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const MaxBrightness(
          child: Text('C'),
        ),
      );

      // Validate that the widget exists
      final childFinder = find.text('C');
      expect(childFinder, findsOneWidget);
    });
  });
}
