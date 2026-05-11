import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/card/delete/argument/delete_card_screen_argument.dart';
import 'package:wallet/src/feature/card/delete/delete_card_screen.dart';

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

  group('DeleteCardScreen - getArgument', () {
    test('returns DeleteCardScreenArgument when valid arguments are passed', () {
      const argument = DeleteCardScreenArgument(attestationId: 'card-123', cardTitle: 'Driving License');
      final settings = RouteSettings(arguments: argument.toMap());

      final result = DeleteCardScreen.getArgument(settings);

      expect(result, argument);
    });

    test('throws UnsupportedError when arguments are null', () {
      const settings = RouteSettings(arguments: null);

      expect(() => DeleteCardScreen.getArgument(settings), throwsA(isA<UnsupportedError>()));
    });

    test('throws UnsupportedError when arguments are invalid', () {
      const settings = RouteSettings(arguments: 'invalid');

      expect(() => DeleteCardScreen.getArgument(settings), throwsA(isA<UnsupportedError>()));
    });
  });
}
