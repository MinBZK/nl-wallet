import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/default_text_and_focus_style.dart';
import 'package:wallet/src/theme/light_wallet_theme.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  const kGoldenSize = Size(360, 180);

  final WidgetStatesController statesControllerDefault = WidgetStatesController();
  final WidgetStatesController statesControllerPressed = WidgetStatesController({WidgetState.pressed});
  final WidgetStatesController statesControllerFocused = WidgetStatesController({WidgetState.focused});

  final TextStyle? textStyleDefault = LightWalletTheme.textTheme.bodyLarge;

  group('goldens', () {
    testGoldens(
      'widget states',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                DefaultTextAndFocusStyle(
                  statesController: statesControllerDefault,
                  textStyle: textStyleDefault,
                  pressedOrFocusedColor: Colors.red,
                  underlineWhenPressedOrFocused: true,
                  child: const Text('Title - Default state'),
                ),
                DefaultTextAndFocusStyle(
                  statesController: statesControllerPressed,
                  textStyle: textStyleDefault,
                  pressedOrFocusedColor: Colors.red,
                  underlineWhenPressedOrFocused: true,
                  child: const Text('Title - Pressed state'),
                ),
                DefaultTextAndFocusStyle(
                  statesController: statesControllerFocused,
                  textStyle: textStyleDefault,
                  pressedOrFocusedColor: Colors.red,
                  underlineWhenPressedOrFocused: true,
                  child: const Text('Title - Focused state'),
                ),
                DefaultTextAndFocusStyle(
                  statesController: statesControllerFocused,
                  textStyle: textStyleDefault,
                  underlineWhenPressedOrFocused: true,
                  child: const Text('Title - Focused state - No color'),
                ),
                DefaultTextAndFocusStyle(
                  statesController: statesControllerFocused,
                  textStyle: textStyleDefault,
                  pressedOrFocusedColor: Colors.red,
                  underlineWhenPressedOrFocused: false,
                  child: const Text('Title - Focused state - No underline'),
                ),
                DefaultTextAndFocusStyle(
                  statesController: statesControllerFocused,
                  textStyle: textStyleDefault,
                  underlineWhenPressedOrFocused: false,
                  child: const Text('Title - Focused state - No color & underline'),
                ),
              ],
            ),
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('default_text_and_focus_style/light');
      },
    );
  });

  group('widgets', () {
    testWidgets('child is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        DefaultTextAndFocusStyle(
          statesController: statesControllerDefault,
          textStyle: textStyleDefault,
          child: const Text('Title'),
        ),
      );

      // Validate that the widget exists
      final textFinder = find.text('Title');
      expect(textFinder, findsOneWidget);
    });
  });

  testWidgets('child is visible when textStyle is null', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      DefaultTextAndFocusStyle(
        statesController: statesControllerDefault,
        textStyle: null,
        child: const Text('Title'),
      ),
    );

    // Validate that the widget exists
    final textFinder = find.text('Title');
    expect(textFinder, findsOneWidget);
  });
}
