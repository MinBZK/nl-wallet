import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/history/detail/argument/history_detail_screen_argument.dart';

import '../../../../mocks/mock_data.dart';

void main() {
  test(
    'serialize to and from Map<> yields identical object',
    () {
      final expected = HistoryDetailScreenArgument(
        timelineAttribute: WalletMockData.interactionTimelineAttribute,
        docType: 'com.example.docType',
      );
      final serialized = expected.toMap();
      final result = HistoryDetailScreenArgument.fromMap(serialized);
      expect(result, expected);
    },
  );
}
