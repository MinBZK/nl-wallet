import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/pin/check_pin_result.dart';

void main() {
  group('CheckPinResult', () {
    group('CheckPinResultIncorrect', () {
      test('supports value equality', () {
        expect(
          CheckPinResultIncorrect(attemptsLeftInRound: 3, isFinalRound: false),
          equals(CheckPinResultIncorrect(attemptsLeftInRound: 3, isFinalRound: false)),
        );
      });

      test('different values are not equal', () {
        expect(
          CheckPinResultIncorrect(attemptsLeftInRound: 3, isFinalRound: false),
          isNot(equals(CheckPinResultIncorrect(attemptsLeftInRound: 2, isFinalRound: false))),
        );
        expect(
          CheckPinResultIncorrect(attemptsLeftInRound: 3, isFinalRound: false),
          isNot(equals(CheckPinResultIncorrect(attemptsLeftInRound: 3, isFinalRound: true))),
        );
      });
    });

    group('CheckPinResultTimeout', () {
      test('supports value equality', () {
        expect(
          CheckPinResultTimeout(timeoutMillis: 1000),
          equals(CheckPinResultTimeout(timeoutMillis: 1000)),
        );
      });

      test('different values are not equal', () {
        expect(
          CheckPinResultTimeout(timeoutMillis: 1000),
          isNot(equals(CheckPinResultTimeout(timeoutMillis: 2000))),
        );
      });
    });

    group('CheckPinResultBlocked', () {
      test('supports value equality', () {
        expect(
          CheckPinResultBlocked(),
          equals(CheckPinResultBlocked()),
        );
      });
    });

    test('different subclasses are not equal', () {
      expect(
        CheckPinResultBlocked(),
        isNot(equals(CheckPinResultTimeout(timeoutMillis: 1000))),
      );
      expect(
        CheckPinResultIncorrect(attemptsLeftInRound: 1),
        isNot(equals(CheckPinResultBlocked())),
      );
    });
  });
}
