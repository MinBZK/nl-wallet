import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/feature/common/widget/attribute/attribute_row.dart';

import '../../../../../wallet_app_test_widget.dart';

/// Note that this test indirectly also verifies:
/// [DataAttributeRow], [RequestedAttributeRow] and [UiAttributeRow]
void main() {
  const kGoldenSize = Size(160, 38);

  group(
    'goldens',
    () {
      testGoldens(
        'light ui attribute',
        (tester) async {
          await tester.pumpWidgetBuilder(
            AttributeRow(
              attribute: UiAttribute.untranslated(
                key: 'key',
                label: 'Label',
                value: const StringValue('Value'),
                icon: Icons.add_card_outlined,
              ),
            ),
            wrapper: walletAppWrapper(brightness: Brightness.light),
            surfaceSize: kGoldenSize,
          );
          await screenMatchesGolden(tester, 'attribute_row/light.ui');
        },
      );
      testGoldens(
        'dark ui attribute',
        (tester) async {
          await tester.pumpWidgetBuilder(
            AttributeRow(
              attribute: UiAttribute.untranslated(
                key: 'key',
                label: 'Label',
                value: const StringValue('Value'),
                icon: Icons.add_card_outlined,
              ),
            ),
            wrapper: walletAppWrapper(brightness: Brightness.dark),
            surfaceSize: kGoldenSize,
          );
          await screenMatchesGolden(tester, 'attribute_row/dark.ui');
        },
      );

      testGoldens(
        'light missing text attribute',
        (tester) async {
          await tester.pumpWidgetBuilder(
            AttributeRow(
              attribute: MissingAttribute.untranslated(
                label: 'Text',
                key: 'mock.other',
              ),
            ),
            wrapper: walletAppWrapper(brightness: Brightness.light),
            surfaceSize: kGoldenSize,
          );
          await screenMatchesGolden(tester, 'attribute_row/light.requested.text');
        },
      );

      testGoldens(
        'light data text attribute',
        (tester) async {
          await tester.pumpWidgetBuilder(
            AttributeRow(
              attribute: DataAttribute.untranslated(
                label: 'Label',
                value: const StringValue('Value'),
                sourceCardDocType: '',
                key: 'mock.other',
              ),
            ),
            wrapper: walletAppWrapper(brightness: Brightness.light),
            surfaceSize: kGoldenSize,
          );
          await screenMatchesGolden(tester, 'attribute_row/light.data.text');
        },
      );
    },
  );

  group('widgets', () {
    testWidgets('Label and value are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        AttributeRow(
          attribute: UiAttribute.untranslated(
            key: 'K',
            label: 'L',
            value: const StringValue('V'),
            icon: Icons.add_card_outlined,
          ),
        ),
      );

      // Validate that the button exists
      final labelFinder = find.text('L');
      final valueFinder = find.text('V');
      expect(labelFinder, findsOneWidget);
      expect(valueFinder, findsOneWidget);
    });
  });
}
