import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/sheet/explanation_sheet.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';

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
        await screenMatchesGolden('explanation_sheet/light');
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
        await screenMatchesGolden('explanation_sheet/dark');
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

    testWidgets('show displays the sheet and close button dismisses it', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) => Scaffold(
            body: ElevatedButton(
              onPressed: () => ExplanationSheet.show(
                context,
                title: 'T',
                description: 'D',
                closeButtonText: 'C',
              ),
              child: const Text('Show Sheet'),
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Sheet'));
      await tester.pumpAndSettle();
      expect(find.text('T'), findsOneWidget);

      await tester.tap(find.text('C'));
      await tester.pumpAndSettle();
      expect(find.text('T'), findsNothing);
    });
  });
}
