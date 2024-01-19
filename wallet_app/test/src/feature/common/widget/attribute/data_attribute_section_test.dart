import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/widget/attribute/data_attribute_section.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mock_data.dart';

void main() {
  const kGoldenSize = Size(350, 180);

  group('goldens', () {
    testGoldens(
      'light text',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          DataAttributeSection(
            sourceCardTitle: 'Card Title',
            attributes: [WalletMockData.textDataAttribute],
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'data_attribute_section/light');
      },
    );

    testGoldens(
      'dark text',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          DataAttributeSection(
            sourceCardTitle: 'Card Title',
            attributes: [WalletMockData.textDataAttribute],
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.dark,
        );
        await screenMatchesGolden(tester, 'data_attribute_section/dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('widgets are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        DataAttributeSection(
          sourceCardTitle: 'Card Title',
          attributes: [WalletMockData.textDataAttribute],
        ),
      );

      // Validate that the widget exists
      final titleFinder = find.textContaining('Card Title');
      final labelFinder = find.text('Label');
      final valueFinder = find.text('Value');
      expect(titleFinder, findsOneWidget);
      expect(labelFinder, findsOneWidget);
      expect(valueFinder, findsOneWidget);
    });
  });
}
