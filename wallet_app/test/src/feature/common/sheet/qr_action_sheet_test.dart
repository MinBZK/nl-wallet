import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:wallet/src/feature/common/sheet/qr_action_sheet.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

const kGoldenSize = Size(390, 341);

void main() {
  group('goldens', () {
    testGoldens(
      'light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const QrActionSheet(),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('qr_action_sheet/light');
      },
    );

    testGoldens(
      'dark',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const QrActionSheet(),
          surfaceSize: kGoldenSize,
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('qr_action_sheet/dark');
      },
    );

    testGoldens(
      'light 2x textScaleSize',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const QrActionSheet(),
          surfaceSize: const Size(390, 695),
          textScaleSize: 2,
        );
        await screenMatchesGolden('qr_action_sheet/light.scaled');
      },
    );
  });

  group('widgets', () {
    testWidgets('title and description are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const QrActionSheet());

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.qrActionSheetTitle), findsOneWidget);
      expect(find.text(l10n.qrActionSheetDescription), findsOneWidget);
    });

    testWidgets('scan option label and description are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const QrActionSheet());

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.qrActionSheetScanQrTitle), findsOneWidget);
      expect(find.text(l10n.qrActionSheetScanQrDescription), findsOneWidget);
    });

    testWidgets('show QR option label and description are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const QrActionSheet());

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.qrActionSheetShowQrTitle), findsOneWidget);
      expect(find.text(l10n.qrActionSheetShowQrDescription), findsOneWidget);
    });

    testWidgets('close button is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const QrActionSheet());

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.generalSheetCloseCta), findsOneWidget);
    });
  });
}
