import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/feature/common/widget/card/shared_attributes_card.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mock_data.dart';
import '../../../../test_util/golden_utils.dart';

const _defaultTestSurfaceSize = Size(328, 208);

void main() {
  group('widgets', () {
    testWidgets('card title is shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        SharedAttributesCard(card: WalletMockData.card, attributes: WalletMockData.card.attributes),
      );

      // Check if the card title is shown
      final titleFinder = find.textContaining(WalletMockData.card.title.testValue);
      expect(titleFinder, findsOneWidget);
    });

    testWidgets('card title includes correct shared attributes count', (tester) async {
      final sharedAttributes = [
        WalletMockData.textDataAttribute,
        WalletMockData.textDataAttribute,
        WalletMockData.textDataAttribute,
        WalletMockData.textDataAttribute,
        WalletMockData.textDataAttribute,
      ];
      await tester.pumpWidgetWithAppWrapper(
        SharedAttributesCard(card: WalletMockData.card, attributes: sharedAttributes),
      );

      // Check if the card title includes the correct amount
      final titleFinder = find.textContaining(sharedAttributes.length.toString());
      expect(titleFinder, findsOneWidget);
    });

    testWidgets('shared attribute titles are shown', (tester) async {
      final sharedAttributes = [
        DataAttribute.untranslated(
          key: 'key1',
          label: 'First label',
          value: const StringValue('Value1'),
          sourceCardDocType: 'com.example.docType',
        ),
        DataAttribute.untranslated(
          key: 'key2',
          label: 'Second label',
          value: const StringValue('Value2'),
          sourceCardDocType: 'com.example.docType',
        ),
      ];
      await tester.pumpWidgetWithAppWrapper(
        SharedAttributesCard(card: WalletMockData.card, attributes: sharedAttributes),
      );

      // Check if the attribute labels are shown
      for (final attribute in sharedAttributes) {
        final attributeFinder = find.text(attribute.label.testValue);
        expect(attributeFinder, findsOneWidget);
      }
    });

    testWidgets('validate click listener', (tester) async {
      bool listenerTriggered = false;
      await tester.pumpWidgetWithAppWrapper(
        SharedAttributesCard(
          card: WalletMockData.card,
          attributes: WalletMockData.card.attributes,
          onTap: () => listenerTriggered = true,
        ),
      );

      // Tap the arrow icon
      final arrowFinder = find.bySubtype<Icon>();
      await tester.tap(arrowFinder);

      // Validate the onTap listener was called
      expect(listenerTriggered, isTrue);
    });
  });

  group('goldens', () {
    testGoldens(
      'shared attributes with simple rendering card - light mode',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          surfaceSize: _defaultTestSurfaceSize,
          Container(
            height: _defaultTestSurfaceSize.height,
            width: _defaultTestSurfaceSize.width,
            padding: const EdgeInsets.only(bottom: 8) /* to allow shadow to render */,
            child: SharedAttributesCard(
              card: WalletMockData.simpleRenderingCard,
              attributes: WalletMockData.simpleRenderingCard.attributes + WalletMockData.simpleRenderingCard.attributes,
              onTap: () {},
            ),
          ),
        );
        await screenMatchesGolden('shared_attributes/simple');
      },
    );
    testGoldens(
      'shared attributes with mock rendering card - dark mode',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          surfaceSize: _defaultTestSurfaceSize,
          Container(
            height: _defaultTestSurfaceSize.height,
            width: _defaultTestSurfaceSize.width,
            padding: const EdgeInsets.only(bottom: 8) /* to allow shadow to render */,
            child: SharedAttributesCard(
              card: WalletMockData.card,
              attributes: WalletMockData.card.attributes,
              onTap: () {},
            ),
          ),
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('shared_attributes/mock');
      },
    );
  });
}
