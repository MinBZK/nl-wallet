import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/card/summary/argument/card_summary_screen_argument.dart';

void main() {
  test(
    'serialize to and from Map<> yields identical object',
    () {
      const expected = CardSummaryScreenArgument(cardId: '567', cardTitle: 'Persoonsgegevens');
      final serialized = expected.toMap();
      final result = CardSummaryScreenArgument.fromMap(serialized);
      expect(result, expected);
    },
  );
}
