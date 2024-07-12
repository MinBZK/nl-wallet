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

  test(
    'hashCode matches on identical objects',
    () {
      const a = CardDataScreenArgument(cardId: '567', cardTitle: 'Persoonsgegevens');
      const b = CardDataScreenArgument(cardId: '567', cardTitle: 'Persoonsgegevens');
      expect(a.hashCode, b.hashCode);
    },
  );

  test(
    'hashCode differs on non identical objects',
    () {
      const a = CardDataScreenArgument(cardId: '567', cardTitle: 'Persoonsgegevens');
      const b = CardDataScreenArgument(cardId: '111', cardTitle: 'Persoonsgegevens');
      expect(a.hashCode, isNot(b.hashCode));
    },
  );
}
