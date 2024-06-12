import 'package:test/test.dart';
import 'package:wallet/src/util/helper/semantics_helper.dart';

void main() {
  group('valid number strings', () {
    test('number starting with 0 is split correctly', () {
      const String input = '01234';
      const String expectedOutput = '0 1 2 3 4';
      expect(SemanticsHelper.splitNumberString(input), expectedOutput);
    });
    test('number starting with 1 is split correctly', () {
      const String input = '1234';
      const String expectedOutput = '1 2 3 4';
      expect(SemanticsHelper.splitNumberString(input), expectedOutput);
    });
    test('single digit number is split correctly', () {
      const String input = '1';
      const String expectedOutput = '1';
      expect(SemanticsHelper.splitNumberString(input), expectedOutput);
    });
    test('two digit number is split correctly', () {
      const String input = '34';
      const String expectedOutput = '3 4';
      expect(SemanticsHelper.splitNumberString(input), expectedOutput);
    });
    test('8 digit number is split correctly', () {
      const String input = '12345678';
      const String expectedOutput = '1 2 3 4 5 6 7 8';
      expect(SemanticsHelper.splitNumberString(input), expectedOutput);
    });
    test('9 digit number is split correctly', () {
      const String input = '123456789';
      const String expectedOutput = '1 2 3 4 5 6 7 8 9';
      expect(SemanticsHelper.splitNumberString(input), expectedOutput);
    });
  });

  group('invalid number strings', () {
    test('number starting with space is not processed', () {
      const String input = ' 12345678';
      expect(SemanticsHelper.splitNumberString(input), input);
    });
    test('number ending with space is not processed', () {
      const String input = '12345678 ';
      expect(SemanticsHelper.splitNumberString(input), input);
    });
    test('negative numbers are not processed', () {
      const String input = '-234';
      expect(SemanticsHelper.splitNumberString(input), input);
    });
    test('number starting with text is not processed', () {
      const String input = 'March 9';
      expect(SemanticsHelper.splitNumberString(input), input);
    });
    test('number containing text is not processed', () {
      const String input = '9th of March 2026';
      expect(SemanticsHelper.splitNumberString(input), input);
    });
    test('number ending with text is not processed', () {
      const String input = '9th of March';
      expect(SemanticsHelper.splitNumberString(input), input);
    });
  });
}
