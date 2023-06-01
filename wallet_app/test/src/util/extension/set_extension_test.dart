import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/util/extension/set_extension.dart';

void main() {
  group('toggle', () {
    late Set<String> set;

    setUp(() {
      set = {'hello', 'world'};
    });

    test('non existing value is added to set', () async {
      set.toggle('!');
      expect(set, {'hello', 'world', '!'});
    });

    test('existing value is removed from set', () async {
      set.toggle('hello');
      expect(set, {'world'});
    });
  });
}
