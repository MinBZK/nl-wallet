import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/card/detail/argument/card_detail_screen_argument.dart';

import '../../../../mocks/mock_data.dart';

void main() {
  group(
    '(de)serialization',
    () {
      test(
        'CardDetailScreenArgument without a full card is (de)serialized correctly',
        () {
          const expected = CardDetailScreenArgument(cardId: '567', cardTitle: 'Persoonsgegevens');
          final serialized = expected.toMap();
          final result = CardDetailScreenArgument.fromMap(serialized);
          expect(result, expected);
        },
      );

      test(
        'CardDetailScreenArgument with a full card is (de)serialized correctly',
        () {
          final expected = CardDetailScreenArgument.forCard(WalletMockData.card);
          final serialized = expected.toMap();
          final result = CardDetailScreenArgument.fromMap(serialized);
          expect(result, expected);
        },
      );
    },
  );
}
