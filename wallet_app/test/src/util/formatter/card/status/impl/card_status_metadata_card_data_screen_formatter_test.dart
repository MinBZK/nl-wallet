import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/card/status/card_status.dart';
import 'package:wallet/src/util/formatter/card/status/impl/card_status_metadata_card_data_screen_formatter.dart';

void main() {
  late CardStatusMetadataCardDataScreenFormatter formatter;

  setUp(() {
    formatter = CardStatusMetadataCardDataScreenFormatter();
  });

  group('show', () {
    test('returns true for validSoon status', () {
      expect(formatter.show(CardStatus.validSoon), true);
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

    test('returns true for unknown status', () {
      expect(formatter.show(CardStatus.unknown), true);
    });
  });
}
