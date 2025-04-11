import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/feature/common/widget/select_card_row.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  const kGoldenSize = Size(350, 97);

  group('goldens', () {
    testGoldens(
      'light select card row',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          SelectCardRow(
            card: WalletMockData.card,
            isSelected: true,
            onCardSelectionToggled: (_) {},
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('select_card_row/light.selected');
      },
    );
    testGoldens(
      'dark select card row',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          SelectCardRow(
            card: WalletMockData.card,
            isSelected: true,
            onCardSelectionToggled: (_) {},
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('select_card_row/dark.selected');
      },
    );
  });

  testGoldens(
    'light select card row unselected',
    (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        SelectCardRow(
          card: WalletMockData.card,
          isSelected: false,
          onCardSelectionToggled: (_) {},
        ),
        surfaceSize: kGoldenSize,
      );
      await screenMatchesGolden('select_card_row/light.unselected');
    },
  );

  testGoldens(
    'light select card row error',
    (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        SelectCardRow(
          card: WalletMockData.card,
          isSelected: true,
          showError: true,
          onCardSelectionToggled: (_) {},
        ),
        surfaceSize: kGoldenSize,
      );
      await screenMatchesGolden('select_card_row/light.selected.error');
    },
  );

  group('widgets', () {
    testWidgets('widgets are visible', (tester) async {
      final testCard = WalletMockData.simpleRenderingCard;
      await tester.pumpWidgetWithAppWrapper(
        SelectCardRow(
          card: testCard,
          isSelected: true,
          onCardSelectionToggled: (_) {},
        ),
      );

      // Validate that the widget exists
      final titleFinder = find.text(testCard.metadata.first.name);
      expect(titleFinder, findsNWidgets(2)); // Once readable, once inside the rendered WalletCard
      // Look for subtitle
      if (testCard.metadata.first.rawSummary != null) {
        final subtitleFinder = find.text(testCard.metadata.first.rawSummary ?? '');
        expect(subtitleFinder, findsNWidgets(2)); // Once readable, once inside the rendered WalletCard
      }
    });

    testWidgets('onCardSelectionToggled fires with the correct id tapped', (tester) async {
      bool isToggled = false;
      String? tappedCardId;
      await tester.pumpWidgetWithAppWrapper(
        SelectCardRow(
          card: WalletMockData.card,
          isSelected: true,
          onCardSelectionToggled: (WalletCard card) {
            isToggled = true;
            tappedCardId = card.id;
          },
        ),
      );

      // Validate that the widget exists
      final checkBoxFinder = find.byType(Checkbox);
      await tester.tap(checkBoxFinder);

      expect(isToggled, isTrue);
      expect(tappedCardId, WalletMockData.card.id);
    });
  });
}
