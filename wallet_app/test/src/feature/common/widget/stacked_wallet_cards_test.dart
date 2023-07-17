import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/widget/stacked_wallet_cards.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/mock_data.dart';

void main() {
  const kGoldenSize = Size(300, 233);

  group('goldens', () {
    testGoldens(
      'light stacked wallet cards',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const StackedWalletCards(
            cards: [
              WalletMockData.cardFront,
              WalletMockData.cardFront,
            ],
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'stacked_wallet_cards/light');
      },
    );
  });

  group('widgets', () {
    testWidgets('card titles are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const StackedWalletCards(
          cards: [
            WalletMockData.cardFront,
            WalletMockData.cardFront,
          ],
        ),
      );

      // Validate that the widget exists
      final widgetFinder = find.text('Sample Card');
      expect(widgetFinder, findsNWidgets(2));
    });
  });
}
