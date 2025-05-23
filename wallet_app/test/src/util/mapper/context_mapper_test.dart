import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/util/mapper/context_mapper.dart';

import '../../mocks/wallet_mocks.mocks.dart';

class TestContextMapper extends ContextMapper<String, int> {
  @override
  // Simple mapping: return the length of the string
  int map(BuildContext context, String input) => input.length;
}

void main() {
  group('ContextMapper', () {
    late TestContextMapper mapper;
    late MockBuildContext mockContext;

    setUp(() {
      mapper = TestContextMapper();
      mockContext = MockBuildContext();
    });

    test('map transforms a single input correctly', () {
      const input = 'test';

      final result = mapper.map(mockContext, input);

      expect(result, 4, reason: "String 'test' should map to length 4");
    });

    test('mapList transforms a list of inputs correctly', () {
      final inputList = ['a', 'ab', 'abc', 'abcd'];

      final results = mapper.mapList(mockContext, inputList);

      expect(results, [1, 2, 3, 4], reason: 'Each string should map to its length');
    });

    test('mapList returns empty list for empty input', () {
      final emptyList = <String>[];

      final results = mapper.mapList(mockContext, emptyList);

      expect(results, isEmpty);
    });
  });
}
