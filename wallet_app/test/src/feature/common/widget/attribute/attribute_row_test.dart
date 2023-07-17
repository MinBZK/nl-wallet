import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/attribute/data_attribute.dart';
import 'package:wallet/src/domain/model/attribute/requested_attribute.dart';
import 'package:wallet/src/domain/model/attribute/ui_attribute.dart';
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
            const AttributeRow(
              attribute: UiAttribute(
                label: 'Label',
                value: 'Value',
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
            const AttributeRow(
              attribute: UiAttribute(
                label: 'Label',
                value: 'Value',
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
        'light requested text attribute',
        (tester) async {
          await tester.pumpWidgetBuilder(
            const AttributeRow(
              attribute: RequestedAttribute(
                name: 'Text',
                type: AttributeType.other,
                valueType: AttributeValueType.text,
              ),
            ),
            wrapper: walletAppWrapper(brightness: Brightness.light),
            surfaceSize: kGoldenSize,
          );
          await screenMatchesGolden(tester, 'attribute_row/light.requested.text');
        },
      );
      testGoldens(
        'light requested image attribute',
        (tester) async {
          await tester.pumpWidgetBuilder(
            const AttributeRow(
              attribute: RequestedAttribute(
                name: 'Image',
                type: AttributeType.other,
                valueType: AttributeValueType.image,
              ),
            ),
            wrapper: walletAppWrapper(brightness: Brightness.light),
            surfaceSize: const Size(160, 90),
          );
          await screenMatchesGolden(tester, 'attribute_row/light.requested.image');
        },
      );

      testGoldens(
        'light data text attribute',
        (tester) async {
          await tester.pumpWidgetBuilder(
            const AttributeRow(
              attribute: DataAttribute(
                label: 'Label',
                value: 'Value',
                sourceCardId: '',
                type: AttributeType.other,
                valueType: AttributeValueType.text,
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
        const AttributeRow(
          attribute: UiAttribute(
            label: 'L',
            value: 'V',
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
