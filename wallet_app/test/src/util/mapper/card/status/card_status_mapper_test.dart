import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/card/status/card_status.dart';
import 'package:wallet/src/util/mapper/card/status/card_status_mapper.dart';
import 'package:wallet_core/core.dart' as core;

void main() {
  late CardStatusMapper mapper;

  setUp(() {
    mapper = CardStatusMapper();
  });

  group('map', () {
    test('null revocationStatus should return CardStatus.unknown', () {
      final CardStatus actual = mapper.map(null);

      expect(actual, CardStatus.unknown);
    });

    test('Valid revocationStatus should return CardStatus.valid', () {
      final CardStatus actual = mapper.map(core.RevocationStatus.Valid);

      expect(actual, CardStatus.valid);
    });

    test('Revoked revocationStatus should return CardStatus.revoked', () {
      final CardStatus actual = mapper.map(core.RevocationStatus.Revoked);

      expect(actual, CardStatus.revoked);
    });

    test('Corrupted revocationStatus should return CardStatus.corrupted', () {
      final CardStatus actual = mapper.map(core.RevocationStatus.Corrupted);

      expect(actual, CardStatus.corrupted);
    });

    test('Undetermined revocationStatus should return CardStatus.unknown', () {
      final CardStatus actual = mapper.map(core.RevocationStatus.Undetermined);

      expect(actual, CardStatus.unknown);
    });
  });
}
