import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/card/detail/argument/card_detail_screen_argument.dart';
import 'package:wallet/src/util/extension/string_extension.dart';

import '../../../../mocks/wallet_mock_data.dart';

void main() {
  group(
    '(de)serialization',
    () {
      test(
        'CardDetailScreenArgument without a full card is (de)serialized correctly',
        () {
          final expected = CardDetailScreenArgument(cardId: '567', cardTitle: 'Persoonsgegevens'.untranslated);
          final serialized = expected.toJson();
          final result = CardDetailScreenArgument.fromJson(serialized);
          expect(result, expected);
        },
      );

      test(
        'CardDetailScreenArgument with a full card is (de)serialized correctly',
        () {
          final expected = CardDetailScreenArgument.fromCard(WalletMockData.card);
          final serialized = expected.toJson();
          final result = CardDetailScreenArgument.fromJson(serialized);
          expect(result, expected);
        },
      );
    },
  );
}
