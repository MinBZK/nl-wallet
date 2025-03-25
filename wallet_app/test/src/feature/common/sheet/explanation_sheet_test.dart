import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/sheet/explanation_sheet.dart';

import '../../../../wallet_app_test_widget.dart';

void main() {
  const kGoldenSize = Size(350, 213);

  group('goldens', () {
    testGoldens(
      'light text',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ExplanationSheet(
            title: 'Title',
            description: 'Description',
            closeButtonText: 'Close',
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'explanation_sheet/light');
      },
    );
    testGoldens(
      'dark text',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ExplanationSheet(
            title: 'Title',
            description: 'Description',
            closeButtonText: 'Close',
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.dark,
        );
        await screenMatchesGolden(tester, 'explanation_sheet/dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('widgets are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ExplanationSheet(
          title: 'T',
          description: 'D',
          closeButtonText: 'C',
        ),
      );

      // Validate that the widget exists
      final titleFinder = find.text('T');
      final descriptionFinder = find.text('D');
      final closeButtonFinder = find.text('C');
      expect(titleFinder, findsOneWidget);
      expect(descriptionFinder, findsOneWidget);
      expect(closeButtonFinder, findsOneWidget);
    });
  });
}
