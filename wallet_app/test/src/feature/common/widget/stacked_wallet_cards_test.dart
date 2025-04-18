import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/feature/common/widget/stacked_wallet_cards.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  const kGoldenSize = Size(300, 233);

  group('goldens', () {
    testGoldens(
      'light stacked wallet cards',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          StackedWalletCards(
            cards: [
              WalletMockData.card,
              WalletMockData.altCard,
            ],
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('stacked_wallet_cards/light');
      },
    );

    testGoldens(
      'light stacked wallet cards with large (3x) font',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          StackedWalletCards(
            cards: [
              WalletMockData.card,
              WalletMockData.altCard,
            ],
          ),
          textScaleSize: 3,
          surfaceSize: const Size(300, 567),
        );
        await screenMatchesGolden('stacked_wallet_cards/light.3x_font');
      },
    );
  });

  group('widgets', () {
    testWidgets('card titles are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        StackedWalletCards(
          cards: [
            WalletMockData.card,
            WalletMockData.card,
          ],
        ),
      );

      // Validate that the widget exists
      final widgetFinder = find.text(WalletMockData.card.title.testValue);
      expect(widgetFinder, findsNWidgets(2));
    });
  });
}
