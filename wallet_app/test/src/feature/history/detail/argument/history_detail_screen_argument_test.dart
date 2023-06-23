import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/history/detail/argument/history_detail_screen_argument.dart';

void main() {
  test(
    'serialize to and from Map<> yields identical object',
    () {
      const expected = HistoryDetailScreenArgument(timelineAttributeId: '123', cardId: '567');
      final serialized = expected.toMap();
      final result = HistoryDetailScreenArgument.fromMap(serialized);
      expect(result, expected);
    },
  );
}
