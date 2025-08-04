import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/feature/common/widget/attribute/data_attribute_row.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../test_util/golden_utils.dart';

void main() {
  group('goldens', () {
    const kGoldenSize = Size(220, 42);

    testGoldens('String', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        DataAttributeRow(
          attribute: DataAttribute.untranslated(
            label: 'StringLabel',
            value: const StringValue('TestString'),
            key: 'mock_string',
          ),
        ),
        surfaceSize: kGoldenSize,
      );
      await screenMatchesGolden('data_attribute_row/string_value');
    });

    testGoldens('String (empty)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        DataAttributeRow(
          attribute: DataAttribute.untranslated(
            label: 'Empty String',
            value: const StringValue(''),
            key: 'mock_string',
          ),
        ),
        surfaceSize: kGoldenSize,
      );
      await screenMatchesGolden('data_attribute_row/string_empty_value');
    });

    testGoldens('Boolean true', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        DataAttributeRow(
          attribute: DataAttribute.untranslated(
            label: 'BoolTrue',
            value: const BooleanValue(true),
            key: 'mock_bool_true',
          ),
        ),
        surfaceSize: kGoldenSize,
      );
      await screenMatchesGolden('data_attribute_row/boolean_value_true');
    });

    testGoldens('Boolean false', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        DataAttributeRow(
          attribute: DataAttribute.untranslated(
            label: 'BoolFalse',
            value: const BooleanValue(false),
            key: 'mock_bool_false',
          ),
        ),
        surfaceSize: kGoldenSize,
      );
      await screenMatchesGolden('data_attribute_row/boolean_value_false');
    });

    testGoldens('Number', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        DataAttributeRow(
          attribute: DataAttribute.untranslated(
            label: 'Number',
            value: const NumberValue(123),
            key: 'mock_number',
          ),
        ),
        surfaceSize: kGoldenSize,
      );
      await screenMatchesGolden('data_attribute_row/number_value');
    });

    testGoldens('Date', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        DataAttributeRow(
          attribute: DataAttribute.untranslated(
            label: 'Date',
            value: DateValue(DateTime(2023, 5, 7)),
            key: 'mock_date',
          ),
        ),
        surfaceSize: kGoldenSize,
      );
      await screenMatchesGolden('data_attribute_row/date_value');
    });

    testGoldens('Null', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        DataAttributeRow(
          attribute: DataAttribute.untranslated(
            label: 'Null',
            value: NullValue(),
            key: 'mock_null',
          ),
        ),
        surfaceSize: kGoldenSize,
      );
      await screenMatchesGolden('data_attribute_row/null_value');
    });

    testGoldens('Empty Array', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        DataAttributeRow(
          attribute: DataAttribute.untranslated(
            label: 'Empty Array',
            value: const ArrayValue([]),
            key: 'mock_array_empty',
          ),
        ),
        surfaceSize: kGoldenSize,
      );
      await screenMatchesGolden('data_attribute_row/array_empty');
    });

    testGoldens('Array (multi value)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        DataAttributeRow(
          attribute: DataAttribute.untranslated(
            label: 'Array',
            value: ArrayValue([
              const StringValue('A'),
              const StringValue('B'),
              const NumberValue(123),
              const BooleanValue(true),
              DateValue(DateTime(2025, 4, 7)),
            ]),
            key: 'array_multi_value',
          ),
        ),
        surfaceSize: const Size(220, 140),
      );
      await screenMatchesGolden('data_attribute_row/array_multi_value');
    });

    testGoldens('Array (single value)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        DataAttributeRow(
          attribute: DataAttribute.untranslated(
            label: 'Array',
            value: const ArrayValue([StringValue('Single')]),
            key: 'mock_array',
          ),
        ),
        surfaceSize: const Size(220, 44),
      );
      await screenMatchesGolden('data_attribute_row/array_single_value');
    });
  });
}
