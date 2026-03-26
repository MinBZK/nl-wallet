import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/qr/scan/widget/flashlight_button.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../test_util/golden_utils.dart';
import '../../../../test_util/test_utils.dart';

const flashlightButtonSize = Size(250, 48);

void main() {
  group('goldens', () {
    testGoldens(
      'ltc7 ltc16 ltc19 light - on',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          FlashlightButton(
            onPressed: () {},
            isOn: true,
          ),
          surfaceSize: flashlightButtonSize,
        );
        await screenMatchesGolden('flashlight_button/light.on');
      },
    );

    testGoldens(
      'ltc7 ltc16 ltc19 light - off',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          FlashlightButton(
            onPressed: () {},
            isOn: false,
          ),
          surfaceSize: flashlightButtonSize,
        );
        await screenMatchesGolden('flashlight_button/light.off');
      },
    );

    testGoldens(
      'ltc7 ltc16 ltc19 light focused',
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
        await screenMatchesGolden('flashlight_button/focused.light');
      },
    );

    testGoldens(
      'ltc7 ltc16 ltc19 light scaled',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          FlashlightButton(
            onPressed: () {},
            isOn: true,
          ),
          surfaceSize: const Size(300, 128),
          textScaleSize: 2,
        );
        await screenMatchesGolden('flashlight_button/scaled.light');
      },
    );

    testGoldens(
      'light - visible background',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          ColoredBox(
            color: Colors.red,
            child: FlashlightButton(
              onPressed: () {},
              isOn: true,
            ),
          ),
          surfaceSize: flashlightButtonSize,
        );
        await screenMatchesGolden('flashlight_button/light.visible_bg');
      },
    );

    testGoldens(
      'ltc7 ltc16 ltc19 dark',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          FlashlightButton(
            onPressed: () {},
            isOn: true,
          ),
          brightness: Brightness.dark,
          surfaceSize: flashlightButtonSize,
        );
        await screenMatchesGolden('flashlight_button/dark');
      },
    );

    testGoldens(
      'ltc7 ltc16 ltc19 dark focused',
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
        await screenMatchesGolden('flashlight_button/focused.dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('ltc7 ltc16 ltc19 button onPressed works', (tester) async {
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
