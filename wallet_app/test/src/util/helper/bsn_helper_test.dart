import 'package:test/test.dart';
import 'package:wallet/src/util/helper/bsn_helper.dart';

void main() {
  group('valid BSNs', () {
    test('111222333 is a valid bsn', () {
      const input = '111222333';
      expect(BsnHelper.isValidBsnFormat(input), isTrue);
    });
    test('123456782 is a valid bsn', () {
      const input = '123456782';
      expect(BsnHelper.isValidBsnFormat(input), isTrue);
    });
    test('83456788 is a valid bsn (8 digits)', () {
      const input = '83456788';
      expect(BsnHelper.isValidBsnFormat(input), isTrue);
    });
  });

  group('invalid BSNs', () {
    test('111222334 is an invalid bsn', () {
      const input = '111222334';
      expect(BsnHelper.isValidBsnFormat(input), isFalse);
    });
    test('0111222333 is an invalid bsn', () {
      const input = '0111222333';
      expect(BsnHelper.isValidBsnFormat(input), isFalse);
    });
    test('1112223330 is an invalid bsn', () {
      const input = '1112223330';
      expect(BsnHelper.isValidBsnFormat(input), isFalse);
    });
    test('3456788 is an invalid bsn (7 digits)', () {
      const input = '3456788';
      expect(BsnHelper.isValidBsnFormat(input), isFalse);
    });
  });
}
