import 'package:test/test.dart';
import 'package:wallet/src/domain/model/attribute/converter/localized_string_converter.dart';
import 'package:wallet_core/core.dart';

void main() {
  const LocalizedStringConverter converter = LocalizedStringConverter();

  test('LocalizedString', () {
    const input = LocalizedString(language: 'nl', value: 'test');
    final json = converter.toJson(input);
    final result = converter.fromJson(json);
    expect(result.value, equals(input.value));
    expect(result.language, equals(input.language));
  });
}
