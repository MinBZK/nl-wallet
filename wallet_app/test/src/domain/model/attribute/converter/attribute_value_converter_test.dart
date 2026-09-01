import 'dart:convert';
import 'dart:typed_data';

import 'package:test/test.dart';
import 'package:wallet/src/domain/model/app_image_data.dart';
import 'package:wallet/src/domain/model/attribute/attribute_value.dart';
import 'package:wallet/src/domain/model/attribute/converter/attribute_value_converter.dart';

void main() {
  const AttributeValueConverter converter = AttributeValueConverter();

  /// One sample per variant, so that adding a variant without a converter case shows up here.
  final samples = <String, AttributeValue>{
    'StringValue': const StringValue('string'),
    'BooleanValue': const BooleanValue(true),
    'NumberValue': const NumberValue(1),
    'NumberValue (fraction)': const NumberValue(12.5),
    'DateValue': DateValue(DateTime(2030, 7, 1)),
    'NullValue': NullValue(),
    'ArrayValue': ArrayValue([const StringValue('A'), DateValue(DateTime(2024, 10, 5))]),
    'ImageValue (memory)': ImageValue(AppMemoryImage(Uint8List.fromList([0, 1, 2, 253, 254, 255]))),
    'ImageValue (svg)': const ImageValue(SvgImage('<svg></svg>')),
    'ImageValue (asset)': const ImageValue(AppAssetImage('assets/logo.png')),
    'MapValue': MapValue({
      'vehicle_category_code': const StringValue('B'),
      'issue_date': DateValue(DateTime(2017, 2, 23)),
    }),
    'MapValue (nested in array)': const ArrayValue([
      MapValue({'code': StringValue('AM')}),
      MapValue({'code': StringValue('B')}),
    ]),
  };

  group('to- and fromJson', () {
    samples.forEach((name, input) {
      test(name, () => expect(converter.fromJson(converter.toJson(input)), input));
    });
  });

  group('survives an encode/decode cycle', () {
    samples.forEach((name, input) {
      test(name, () {
        final encoded = jsonEncode(converter.toJson(input));
        final decoded = jsonDecode(encoded) as Map<String, dynamic>;
        expect(converter.fromJson(decoded), input);
      });
    });
  });

  test('a map keeps the order of its keys', () {
    final input = MapValue({
      'vehicle_category_code': const StringValue('B'),
      'issue_date': DateValue(DateTime(2017, 2, 23)),
      'expiry_date': DateValue(DateTime(2024, 10, 20)),
    });
    final result = converter.fromJson(jsonDecode(jsonEncode(converter.toJson(input))) as Map<String, dynamic>);
    expect((result as MapValue).value.keys, input.value.keys);
  });

  test('decoding an unsupported type throws', () {
    expect(
      () => converter.fromJson({'type': 'non-existent-type'}),
      throwsA(isA<UnsupportedError>()),
    );
  });
}
