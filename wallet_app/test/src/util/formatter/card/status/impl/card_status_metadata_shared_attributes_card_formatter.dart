import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/card/status/card_status.dart';
import 'package:wallet/src/util/formatter/card/status/impl/card_status_metadata_shared_attributes_card_formatter.dart';

void main() {
  final mockDateTime = DateTime.now();

  late CardStatusMetadataSharedAttributesCardFormatter formatter;

  setUp(() {
    formatter = CardStatusMetadataSharedAttributesCardFormatter();
  });

  group('show', () {
    test('returns false for validSoon status', () {
      expect(formatter.show(CardStatusValidSoon(validFrom: mockDateTime)), false);
    });

    test('returns false for valid status', () {
      expect(formatter.show(CardStatusValid(validUntil: mockDateTime)), false);
    });

    test('returns false for expiresSoon status', () {
      expect(formatter.show(CardStatusExpiresSoon(validUntil: mockDateTime)), false);
    });

    test('returns true for expired status', () {
      expect(formatter.show(CardStatusExpired(validUntil: mockDateTime)), true);
    });

    test('returns true for revoked status', () {
      expect(formatter.show(const CardStatusRevoked()), true);
    });

    test('returns true for corrupted status', () {
      expect(formatter.show(const CardStatusCorrupted()), true);
    });

    test('returns false for undetermined status', () {
      expect(formatter.show(const CardStatusUndetermined()), false);
    });
  });
}
