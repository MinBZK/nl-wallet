import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/feature/common/widget/stacked_wallet_cards.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';

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
              WalletMockData.card,
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
        StackedWalletCards(
          cards: [
            WalletMockData.card,
            WalletMockData.card,
          ],
        ),
      );

      // Validate that the widget exists
      final widgetFinder = find.text(WalletMockData.card.front.title.testValue);
      expect(widgetFinder, findsNWidgets(2));
    });
  });
}
