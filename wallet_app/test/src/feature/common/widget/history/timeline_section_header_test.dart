import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/widget/history/timeline_section_header.dart';

import '../../../../../wallet_app_test_widget.dart';

void main() {
  const kGoldenSize = Size(150, 37);

  group('goldens', () {
    testGoldens(
      'light header',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          TimelineSectionHeader(dateTime: DateTime(2023, 1, 1)),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'timeline_section_header/light');
      },
    );
    testGoldens(
      'dark header',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          TimelineSectionHeader(dateTime: DateTime(2023, 5, 9)),
          brightness: Brightness.dark,
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'timeline_section_header/dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('Date is rendered as "January 2023"', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        TimelineSectionHeader(dateTime: DateTime(2023, 1, 1)),
      );

      // Validate that the widget exists
      final widgetFinder = find.text('January 2023');
      expect(widgetFinder, findsOneWidget);
    });
  });
}
