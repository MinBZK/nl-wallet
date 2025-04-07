import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/feature/common/widget/attribute/attribute_row.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../test_util/golden_utils.dart';

/// Note that this test indirectly also verifies:
/// [DataAttributeRow], [RequestedAttributeRow] and [UiAttributeRow]
void main() {
  const kGoldenSize = Size(160, 42);

  group(
    'goldens',
    () {
      testGoldens(
        'light ui attribute',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            AttributeRow(
              attribute: UiAttribute.untranslated(
                key: 'key',
                label: 'Label',
                value: const StringValue('Value'),
                icon: Icons.add_card_outlined,
              ),
            ),
            surfaceSize: kGoldenSize,
          );
          await screenMatchesGolden('attribute_row/light.ui');
        },
      );
      testGoldens(
        'dark ui attribute',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            AttributeRow(
              attribute: UiAttribute.untranslated(
                key: 'key',
                label: 'Label',
                value: const StringValue('Value'),
                icon: Icons.add_card_outlined,
              ),
            ),
            brightness: Brightness.dark,
            surfaceSize: kGoldenSize,
          );
          await screenMatchesGolden('attribute_row/dark.ui');
        },
      );

      testGoldens(
        'light missing text attribute',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            AttributeRow(
              attribute: MissingAttribute.untranslated(
                label: 'Text',
                key: 'mock.other',
              ),
            ),
            surfaceSize: kGoldenSize,
          );
          await screenMatchesGolden('attribute_row/light.requested.text');
        },
      );

      testGoldens(
        'light data text attribute',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            AttributeRow(
              attribute: DataAttribute.untranslated(
                label: 'Label',
                value: const StringValue('Value'),
                sourceCardDocType: '',
                key: 'mock.other',
              ),
            ),
            surfaceSize: kGoldenSize,
          );
          await screenMatchesGolden('attribute_row/light.data.text');
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
