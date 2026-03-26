import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/button/qr_action_button.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../test_util/golden_utils.dart';
import '../../../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens(
      'light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          QrActionButton(onPressed: () {}),
          surfaceSize: qrActionButtonSize,
        );
        await screenMatchesGolden('qr_action_button/light');
      },
    );

    testGoldens(
      'light focused',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          QrActionButton(onPressed: () {}),
          surfaceSize: qrActionButtonSize,
        );
        await tester.sendKeyEvent(LogicalKeyboardKey.tab); // Trigger focused state
        await tester.pumpAndSettle();
        await screenMatchesGolden('qr_action_button/light.focused');
      },
    );

    testGoldens(
      'light 2x textScaleSize',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          QrActionButton(onPressed: () {}),
          surfaceSize: const Size(240, 320),
          textScaleSize: 2,
        );
        await screenMatchesGolden('qr_action_button/light.scaled');
      },
    );

    testGoldens(
      'light 4x textScaleSize',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          QrActionButton(onPressed: () {}),
          surfaceSize: const Size(240, 800),
          textScaleSize: 4,
        );
        await screenMatchesGolden('qr_action_button/light.4x_scaled');
      },
    );

    testGoldens(
      'dark',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          QrActionButton(onPressed: () {}),
          surfaceSize: qrActionButtonSize,
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('qr_action_button/dark');
      },
    );

    testGoldens(
      'dark - focused',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          QrActionButton(onPressed: () {}),
          surfaceSize: qrActionButtonSize,
          brightness: Brightness.dark,
        );
        await tester.sendKeyEvent(LogicalKeyboardKey.tab); // Trigger focused state
        await tester.pumpAndSettle();
        await screenMatchesGolden('qr_action_button/dark.focused');
      },
    );
  });

  group('widgets', () {
    testWidgets('widget is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        QrActionButton(onPressed: () {}),
      );

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.qrActionButtonTitle), findsOneWidget);
      expect(find.text(l10n.qrActionButtonSubtitle), findsOneWidget);
    });

    testWidgets('onPressed is triggered when tapped', (tester) async {
      bool pressed = false;
      await tester.pumpWidgetWithAppWrapper(
        QrActionButton(onPressed: () => pressed = true),
      );

      final l10n = await TestUtils.englishLocalizations;
      await tester.tap(find.text(l10n.qrActionButtonTitle));
      expect(pressed, isTrue);
    });
  });
}
