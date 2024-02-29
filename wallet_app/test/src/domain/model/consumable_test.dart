import 'package:test/test.dart';
import 'package:wallet/src/domain/model/consumable.dart';

void main() {
  test('unconsumed value should be available', () {
    const value = 'test';
    final wrapped = Consumable(value);
    expect(value, wrapped.value);
  });

  test('consumed value should return null after consuming', () {
    const value = 'test';
    final wrapped = Consumable(value);
    wrapped.value; // consume the value
    expect(wrapped.value, null);
  });

  test('peeking value should not consume it', () {
    const value = 'test';
    final wrapped = Consumable(value);
    expect(value, wrapped.peek());
    expect(value, wrapped.value);
  });
}
