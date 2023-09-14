import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/card/detail/argument/card_detail_screen_argument.dart';

void main() {
  test(
    'serialize to and from Map<> yields identical object',
    () {
      const expected = CardDetailScreenArgument(cardId: '567', cardTitle: 'Persoonsgegevens');
      final serialized = expected.toMap();
      final result = CardDetailScreenArgument.fromMap(serialized);
      expect(result, expected);
    },
  );
}
