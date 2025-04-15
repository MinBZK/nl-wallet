import 'package:test/test.dart';
import 'package:wallet/src/domain/model/attribute/converter/localized_text_converter.dart';
import 'package:wallet/src/domain/model/localized_text.dart';

void main() {
  const LocalizedTextConverter converter = LocalizedTextConverter();

  test('LocalizedText to- and fromJson', () {
    final LocalizedText input = {
      Locale('en'): 'English',
      Locale('en', 'US'): 'American',
      Locale('nl'): 'dutch',
      Locale('nl', 'BE'): 'Flemish',
    };
    final json = converter.toJson(input);
    final result = converter.fromJson(json);
    expect(result, equals(input));
  });
}
