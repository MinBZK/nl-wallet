import 'package:test/test.dart';
import 'package:wallet/src/domain/model/attribute/attribute_value.dart';
import 'package:wallet/src/domain/model/attribute/converter/attribute_value_converter.dart';
import 'package:wallet/src/domain/model/attribute/value/gender.dart';

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

  test('DateValue', () {
    final input = DateValue(DateTime.fromMillisecondsSinceEpoch(123456789));
    final json = converter.toJson(input);
    final result = converter.fromJson(json);
    expect(result, equals(input));
  });

  test('GenderValue', () {
    const input = GenderValue(Gender.female);
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
