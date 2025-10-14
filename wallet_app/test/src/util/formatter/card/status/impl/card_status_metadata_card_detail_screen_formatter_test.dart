import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/card/status/card_status.dart';
import 'package:wallet/src/util/formatter/card/status/impl/card_status_metadata_card_detail_screen_formatter.dart';

void main() {
  late CardStatusMetadataCardDetailScreenFormatter formatter;

  setUp(() {
    formatter = CardStatusMetadataCardDetailScreenFormatter();
  });

  group('show', () {
    test('returns true for all card statuses', () {
      for (final status in CardStatus.values) {
        expect(formatter.show(status), true, reason: 'Failed for status: $status');
      }
    });
  });
}
