import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/card/delete/argument/delete_card_screen_argument.dart';

void main() {
  group('DeleteCardScreenArgument', () {
    test('toMap and fromMap roundtrip preserves values', () {
      const argument = DeleteCardScreenArgument(attestationId: 'card-123', cardTitle: 'Test Card');
      final map = argument.toMap();
      final restored = DeleteCardScreenArgument.fromMap(map);

      expect(restored.attestationId, 'card-123');
      expect(restored.cardTitle, 'Test Card');
      expect(restored, argument);
    });

    test('equality works correctly', () {
      const a = DeleteCardScreenArgument(attestationId: 'card-123', cardTitle: 'Test Card');
      const b = DeleteCardScreenArgument(attestationId: 'card-123', cardTitle: 'Test Card');
      const c = DeleteCardScreenArgument(attestationId: 'card-456', cardTitle: 'Other Card');

      expect(a, b);
      expect(a, isNot(c));
      expect(a.hashCode, b.hashCode);
    });
  });
}
