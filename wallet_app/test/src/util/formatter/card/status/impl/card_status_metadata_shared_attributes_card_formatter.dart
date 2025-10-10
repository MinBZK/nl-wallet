import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/card/status/card_status.dart';
import 'package:wallet/src/util/formatter/card/status/impl/card_status_metadata_shared_attributes_card_formatter.dart';

void main() {
  late CardStatusMetadataSharedAttributesCardFormatter formatter;

  setUp(() {
    formatter = CardStatusMetadataSharedAttributesCardFormatter();
  });

  group('show', () {
    test('returns false for validSoon status', () {
      expect(formatter.show(CardStatus.validSoon), false);
    });

    test('returns false for valid status', () {
      expect(formatter.show(CardStatus.valid), false);
    });

    test('returns false for expiresSoon status', () {
      expect(formatter.show(CardStatus.expiresSoon), false);
    });

    test('returns true for expired status', () {
      expect(formatter.show(CardStatus.expired), true);
    });

    test('returns true for revoked status', () {
      expect(formatter.show(CardStatus.revoked), true);
    });

    test('returns true for corrupted status', () {
      expect(formatter.show(CardStatus.corrupted), true);
    });

    test('returns false for unknown status', () {
      expect(formatter.show(CardStatus.unknown), false);
    });
  });
}
