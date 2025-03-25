import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/sheet/confirm_action_sheet.dart';
import 'package:wallet/src/feature/common/widget/text/body_text.dart';

import '../../../../wallet_app_test_widget.dart';

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
            cancelButtonText: 'Cancel CTA',
            confirmButtonText: 'Confirm CTA',
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'confirm_action_sheet/light');
      },
    );
    testGoldens(
      'dark confirm action sheet',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ConfirmActionSheet(
            title: 'Title',
            description: 'Description',
            cancelButtonText: 'Cancel CTA',
            confirmButtonText: 'Confirm CTA',
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.dark,
        );
        await screenMatchesGolden(tester, 'confirm_action_sheet/dark');
      },
    );
    testGoldens(
      'light confirm action sheet with extra content and icons and custom color',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          ConfirmActionSheet(
            title: 'Title',
            description: 'Description',
            cancelButtonText: 'No',
            confirmButtonText: 'Yes',
            extraContent: Container(
              color: Colors.black12,
              padding: const EdgeInsets.all(16),
              child: const BodyText(
                'All content within this gray box is extra content. The `extraContent` widget is responsible for its own margins.',
              ),
            ),
            cancelIcon: Icons.close,
            confirmIcon: Icons.check,
            confirmButtonColor: Colors.green,
          ),
          surfaceSize: const Size(350, 342),
        );
        await screenMatchesGolden(tester, 'confirm_action_sheet/light.custom');
      },
    );
  });

  group('widgets', () {
    testWidgets('widgets are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ConfirmActionSheet(
          title: 'T',
          description: 'D',
          cancelButtonText: 'CancelCTA',
          confirmButtonText: 'ConfirmCTA',
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
          cancelButtonText: 'CancelCTA',
          confirmButtonText: 'ConfirmCTA',
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
