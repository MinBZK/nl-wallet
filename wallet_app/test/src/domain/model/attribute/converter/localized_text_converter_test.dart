import 'package:test/test.dart';
import 'package:wallet/src/domain/model/attribute/converter/localized_text_converter.dart';
import 'package:wallet/src/domain/model/localized_text.dart';

void main() {
  const LocalizedTextConverter converter = LocalizedTextConverter();

  test('LocalizedText to- and fromJson', () {
    final LocalizedText input = {
      const Locale('en'): 'English',
      const Locale('en', 'US'): 'American',
      const Locale('nl'): 'dutch',
      const Locale('nl', 'BE'): 'Flemish',
    };
    final json = converter.toJson(input);
    final result = converter.fromJson(json);
    expect(result, equals(input));
  });
}
