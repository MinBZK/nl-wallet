import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/button/scan_qr_button.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../test_util/golden_utils.dart';
import '../../../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens(
      'light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          ScanQrButton(onPressed: () {}),
          surfaceSize: scanQrButtonSize,
        );
        await screenMatchesGolden('scan_qr_button/light');
      },
    );

    testGoldens(
      'light focused',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          ScanQrButton(onPressed: () {}),
          surfaceSize: scanQrButtonSize,
        );
        await tester.sendKeyEvent(LogicalKeyboardKey.tab); // Trigger focused state
        await tester.pumpAndSettle();
        await screenMatchesGolden('scan_qr_button/light.focused');
      },
    );

    testGoldens(
      'light 2x textScaleSize',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          ScanQrButton(onPressed: () {}),
          surfaceSize: scanQrButtonSize,
          textScaleSize: 2,
        );
        await screenMatchesGolden('scan_qr_button/light.scaled');
      },
    );

    testGoldens(
      'light 4x textScaleSize',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          ScanQrButton(onPressed: () {}),
          surfaceSize: const Size(240, 320),
          textScaleSize: 4,
        );
        await screenMatchesGolden('scan_qr_button/light.4x_scaled');
      },
    );

    testGoldens(
      'dark',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          ScanQrButton(onPressed: () {}),
          surfaceSize: scanQrButtonSize,
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('scan_qr_button/dark');
      },
    );

    testGoldens(
      'dark - focused',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          ScanQrButton(onPressed: () {}),
          surfaceSize: scanQrButtonSize,
          brightness: Brightness.dark,
        );
        await tester.sendKeyEvent(LogicalKeyboardKey.tab); // Trigger focused state
        await tester.pumpAndSettle();
        await screenMatchesGolden('scan_qr_button/dark.focused');
      },
    );
  });

  group('widgets', () {
    testWidgets('widget is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        ScanQrButton(onPressed: () {}),
      );

      final l10n = await TestUtils.englishLocalizations;
      final widgetFinder = find.text(l10n.scanQrButtonCta);
      expect(widgetFinder, findsOneWidget);
    });
  });
}
