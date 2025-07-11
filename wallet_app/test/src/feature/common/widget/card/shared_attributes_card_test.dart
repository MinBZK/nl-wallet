import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/feature/common/widget/card/shared_attributes_card.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mock_data.dart';
import '../../../../test_util/golden_utils.dart';

const _defaultTestSurfaceSize = Size(328, 208);
const _largeTestSurfaceSize = Size(328, 260);

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
          sourceCardId: 'com.example.docType',
        ),
        DataAttribute.untranslated(
          key: 'key2',
          label: 'Second label',
          value: const StringValue('Value2'),
          sourceCardId: 'com.example.docType',
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
          onPressed: () => listenerTriggered = true,
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
              onPressed: () {},
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
              onPressed: () {},
            ),
          ),
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('shared_attributes/mock');
      },
    );
    testGoldens(
      'shared attributes with change card button - light mode',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          surfaceSize: _largeTestSurfaceSize,
          Container(
            height: _largeTestSurfaceSize.height,
            width: _largeTestSurfaceSize.width,
            padding: const EdgeInsets.only(bottom: 8) /* to allow shadow to render */,
            child: SharedAttributesCard(
              card: WalletMockData.card,
              attributes: WalletMockData.card.attributes,
              onPressed: () {},
              onChangeCardPressed: () {},
            ),
          ),
        );
        await screenMatchesGolden('shared_attributes/change_card.light');
      },
    );
    testGoldens(
      'shared attributes with change card button - dark mode',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          surfaceSize: _largeTestSurfaceSize,
          Container(
            height: _largeTestSurfaceSize.height,
            width: _largeTestSurfaceSize.width,
            padding: const EdgeInsets.only(bottom: 8),
            child: SharedAttributesCard(
              card: WalletMockData.card,
              attributes: WalletMockData.card.attributes,
              onPressed: () {},
              onChangeCardPressed: () {},
            ),
          ),
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('shared_attributes/change_card.dark');
      },
    );

    testGoldens(
      'shared attributes with simple rendering card - light mode - focused',
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
              onPressed: () {},
            ),
          ),
        );
        await tester.sendKeyEvent(LogicalKeyboardKey.tab);
        await tester.pumpAndSettle();
        await screenMatchesGolden('shared_attributes/simple.focused');
      },
    );

    testGoldens(
      'shared attributes with change card button - light mode - focus on top section',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          surfaceSize: _largeTestSurfaceSize,
          Container(
            height: _largeTestSurfaceSize.height,
            width: _largeTestSurfaceSize.width,
            padding: const EdgeInsets.only(bottom: 8) /* to allow shadow to render */,
            child: SharedAttributesCard(
              card: WalletMockData.simpleRenderingCard,
              attributes: WalletMockData.simpleRenderingCard.attributes,
              onPressed: () {},
              onChangeCardPressed: () {},
            ),
          ),
        );
        await tester.sendKeyEvent(LogicalKeyboardKey.tab);
        await tester.pumpAndSettle();
        await screenMatchesGolden('shared_attributes/change_card.focused.top');
      },
    );

    testGoldens(
      'shared attributes with change card button - light mode - focus on bottom section',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          surfaceSize: _largeTestSurfaceSize,
          Container(
            height: _largeTestSurfaceSize.height,
            width: _largeTestSurfaceSize.width,
            padding: const EdgeInsets.only(bottom: 8) /* to allow shadow to render */,
            child: SharedAttributesCard(
              card: WalletMockData.simpleRenderingCard,
              attributes: WalletMockData.simpleRenderingCard.attributes,
              onPressed: () {},
              onChangeCardPressed: () {},
            ),
          ),
        );
        await tester.sendKeyEvent(LogicalKeyboardKey.tab);
        await tester.sendKeyEvent(LogicalKeyboardKey.tab);
        await tester.pumpAndSettle();
        await screenMatchesGolden('shared_attributes/change_card.focused.bottom');
      },
    );
  });
}
