import 'package:test/test.dart';
import 'package:wallet/src/domain/model/attribute/attribute_value.dart';
import 'package:wallet/src/domain/model/attribute/converter/attribute_value_converter.dart';

void main() {
  const AttributeValueConverter converter = AttributeValueConverter();

  test('StringValue', () {
    const input = StringValue('string');
    final json = converter.toJson(input);
    final result = converter.fromJson(json);
    expect(result, equals(input));
  });

  test('BooleanValue', () {
    const input = BooleanValue(true);
    final json = converter.toJson(input);
    final result = converter.fromJson(json);
    expect(result, equals(input));
  });

  test('NumberValue', () {
    const input = NumberValue(1);
    final json = converter.toJson(input);
    final result = converter.fromJson(json);
    expect(result, equals(input));
  });

  test('decoding an unsupported type throws', () {
    expect(
      () => converter.fromJson({'type': 'non-existent-type'}),
      throwsA(isA<UnsupportedError>()),
    );
  });
}
