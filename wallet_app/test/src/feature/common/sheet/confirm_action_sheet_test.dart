import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/sheet/confirm_action_sheet.dart';
import 'package:wallet/src/feature/common/widget/text/body_text.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  const kGoldenSize = Size(350, 221);

  group('goldens', () {
    testGoldens(
      'light confirm action sheet',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ConfirmActionSheet(
            title: 'Title',
            description: 'Description',
            confirmButton: ConfirmSheetButtonStyle(cta: 'Confirm CTA'),
            cancelButton: ConfirmSheetButtonStyle(cta: 'Cancel CTA'),
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('confirm_action_sheet/light');
      },
    );
    testGoldens(
      'dark confirm action sheet',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ConfirmActionSheet(
            title: 'Title',
            description: 'Description',
            confirmButton: ConfirmSheetButtonStyle(cta: 'Confirm CTA'),
            cancelButton: ConfirmSheetButtonStyle(cta: 'Cancel CTA'),
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('confirm_action_sheet/dark');
      },
    );

    testGoldens(
      'light confirm action sheet with extra content and icons and custom color',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          ConfirmActionSheet(
            title: 'Title',
            description: 'Description',
            confirmButton: const ConfirmSheetButtonStyle(cta: 'Yes', icon: Icons.check, color: Colors.green),
            cancelButton: const ConfirmSheetButtonStyle(cta: 'No', icon: Icons.close),
            extraContent: Container(
              color: Colors.black12,
              padding: const EdgeInsets.all(16),
              child: const BodyText(
                'All content within this gray box is extra content. The `extraContent` widget is responsible for its own margins.',
              ),
            ),
          ),
          surfaceSize: const Size(350, 342),
        );
        await screenMatchesGolden('confirm_action_sheet/light.custom');
      },
    );

    testGoldens(
      'dark confirm action sheet with custom cancel color',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ConfirmActionSheet(
            title: 'Title',
            description: 'Description',
            confirmButton: ConfirmSheetButtonStyle(cta: 'Yes'),
            cancelButton: ConfirmSheetButtonStyle(cta: 'No', color: Colors.white),
          ),
          brightness: Brightness.dark,
          surfaceSize: const Size(350, 217),
        );
        await screenMatchesGolden('confirm_action_sheet/dark.custom');
      },
    );
  });

  group('widgets', () {
    testWidgets('widgets are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ConfirmActionSheet(
          title: 'T',
          description: 'D',
          confirmButton: ConfirmSheetButtonStyle(cta: 'ConfirmCTA'),
          cancelButton: ConfirmSheetButtonStyle(cta: 'CancelCTA'),
          extraContent: Text('EC'),
        ),
      );

      // Validate that the widget exists
      final titleFinder = find.text('T');
      final descriptionFinder = find.text('D');
      final cancelCtaFinder = find.text('CancelCTA');
      final confirmCtaFinder = find.text('ConfirmCTA');
      final extraContentFinder = find.text('EC');
      expect(titleFinder, findsOneWidget);
      expect(descriptionFinder, findsOneWidget);
      expect(cancelCtaFinder, findsOneWidget);
      expect(confirmCtaFinder, findsOneWidget);
      expect(extraContentFinder, findsOneWidget);
    });

    testWidgets('onPressed callbacks are triggered', (tester) async {
      bool confirmPressed = false;
      bool cancelPressed = false;
      await tester.pumpWidgetWithAppWrapper(
        ConfirmActionSheet(
          title: 'T',
          description: 'D',
          confirmButton: const ConfirmSheetButtonStyle(cta: 'ConfirmCTA'),
          cancelButton: const ConfirmSheetButtonStyle(cta: 'CancelCTA'),
          onConfirmPressed: () => confirmPressed = true,
          onCancelPressed: () => cancelPressed = true,
        ),
      );

      // Validate that the widget exists
      final cancelCtaFinder = find.text('CancelCTA');
      final confirmCtaFinder = find.text('ConfirmCTA');
      await tester.tap(cancelCtaFinder);
      await tester.tap(confirmCtaFinder);

      expect(cancelPressed, isTrue);
      expect(confirmPressed, isTrue);
    });
  });
}
