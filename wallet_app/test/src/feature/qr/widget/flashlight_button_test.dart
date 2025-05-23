import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/qr/widget/flashlight_button.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

const flashlightButtonSize = Size(250, 48);

void main() {
  group('goldens', () {
    testGoldens(
      'light - on',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          FlashlightButton(
            onPressed: () {},
            isOn: true,
          ),
          surfaceSize: flashlightButtonSize,
        );
        await screenMatchesGolden('flashlight/light.on');
      },
    );

    testGoldens(
      'light - off',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          FlashlightButton(
            onPressed: () {},
            isOn: false,
          ),
          surfaceSize: flashlightButtonSize,
        );
        await screenMatchesGolden('flashlight/light.off');
      },
    );

    testGoldens(
      'light focused',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          FlashlightButton(
            onPressed: () {},
            isOn: true,
          ),
          surfaceSize: flashlightButtonSize,
        );
        await tester.sendKeyEvent(LogicalKeyboardKey.tab); // Trigger focused state
        await tester.pumpAndSettle();
        await screenMatchesGolden('flashlight/focused.light');
      },
    );

    testGoldens(
      'light scaled',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          FlashlightButton(
            onPressed: () {},
            isOn: true,
          ),
          surfaceSize: Size(300, 128),
          textScaleSize: 2,
        );
        await screenMatchesGolden('flashlight/scaled.light');
      },
    );

    testGoldens(
      'dark',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          FlashlightButton(
            onPressed: () {},
            isOn: true,
          ),
          brightness: Brightness.dark,
          surfaceSize: flashlightButtonSize,
        );
        await screenMatchesGolden('flashlight/dark');
      },
    );

    testGoldens(
      'dark focused',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          FlashlightButton(
            onPressed: () {},
            isOn: true,
          ),
          brightness: Brightness.dark,
          surfaceSize: flashlightButtonSize,
        );
        await tester.sendKeyEvent(LogicalKeyboardKey.tab); // Trigger focused state
        await tester.pumpAndSettle();
        await screenMatchesGolden('flashlight/focused.dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('button onPressed works', (tester) async {
      bool isPressed = false;
      await tester.pumpWidgetWithAppWrapper(
        FlashlightButton(
          onPressed: () => isPressed = true,
          isOn: true,
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      final widgetFinder = find.text(l10n.qrScreenDisableTorchCta);

      expect(isPressed, isFalse, reason: 'button callback should not yet be triggered');
      // Tap the button
      await tester.tap(widgetFinder);
      expect(isPressed, isTrue, reason: 'button callback should be triggered');
    });
  });
}
