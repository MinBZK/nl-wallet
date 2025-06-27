import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/select_card_row.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  const kGoldenSize = Size(350, 80);

  group('goldens', () {
    testGoldens(
      'light select card row',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          SelectCardRow(
            card: WalletMockData.card,
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('select_card_row/light');
      },
    );
    testGoldens(
      'dark select card row',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          SelectCardRow(
            card: WalletMockData.altCard,
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('select_card_row/dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('widgets are visible', (tester) async {
      final testCard = WalletMockData.simpleRenderingCard;
      await tester.pumpWidgetWithAppWrapper(
        SelectCardRow(
          card: testCard,
          onPressed: () {},
        ),
      );

      // Validate that the widget exists
      final titleFinder = find.text(testCard.metadata.first.name);
      expect(titleFinder, findsOneWidget);
      // Look for subtitle
      if (testCard.metadata.first.rawSummary != null) {
        final subtitleFinder = find.text(testCard.metadata.first.rawSummary ?? '');
        expect(subtitleFinder, findsOneWidget);
      }
    });

    testWidgets('onCardSelectionToggled fires with the correct id tapped', (tester) async {
      bool isPressed = false;
      await tester.pumpWidgetWithAppWrapper(
        SelectCardRow(
          card: WalletMockData.card,
          onPressed: () => isPressed = true,
        ),
      );

      // Validate that the widget exists
      final itemFinder = find.byType(SelectCardRow);
      await tester.tap(itemFinder);

      expect(isPressed, isTrue);
    });
  });
}
