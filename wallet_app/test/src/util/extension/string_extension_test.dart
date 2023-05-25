import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/util/extension/string_extension.dart';

void main() {
  group('capitalize', () {
    test('lowercase string returns capitalized first character', () async {
      expect('hello world'.capitalize(), 'Hello world');
    });

    test('capitalized string returns unmodified', () async {
      expect('Hello world'.capitalize(), 'Hello world');
    });

    test('empty string returns empty string', () async {
      expect(''.capitalize(), '');
    });
  });

  group('removeLastChar', () {
    test('non-empty string returns removed last character', () async {
      expect('Hello world'.removeLastChar(), 'Hello worl');
    });

    test('empty string returns empty string', () async {
      expect(''.removeLastChar(), '');
    });
  });

  group('addSpaceSuffix', () {
    test('non-empty string returns added space suffix', () async {
      expect('Hello world'.addSpaceSuffix(), 'Hello world ');
    });

    test('empty string returns empty string', () async {
      expect(''.addSpaceSuffix(), '');
    });
  });
}
