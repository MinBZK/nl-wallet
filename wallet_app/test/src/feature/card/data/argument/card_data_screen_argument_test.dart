import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/card/data/argument/card_data_screen_argument.dart';

void main() {
  test(
    'serialize to and from Map<> yields identical object',
    () {
      const expected = CardDataScreenArgument(cardId: '567', cardTitle: 'Persoonsgegevens');
      final serialized = expected.toMap();
      final result = CardDataScreenArgument.fromMap(serialized);
      expect(result, expected);
    },
  );
}
