import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/review_revocation_code_screen/bloc/review_revocation_code_bloc.dart';

void main() {
  group('ReviewRevocationCodeEvent', () {
    group('ReviewRevocationCodeRequested', () {
      test('supports equals', () {
        expect(
          const ReviewRevocationCodeRequested(),
          const ReviewRevocationCodeRequested(),
        );
      });
    });

    group('ReviewRevocationCodeLoaded', () {
      test('supports equals', () {
        expect(
          const ReviewRevocationCodeLoaded('code'),
          const ReviewRevocationCodeLoaded('code'),
        );
      });

      test('returns false for different codes', () {
        expect(
          const ReviewRevocationCodeLoaded('code1'),
          isNot(const ReviewRevocationCodeLoaded('code2')),
        );
      });
    });

    group('ReviewRevocationCodeRestartFlow', () {
      test('supports equals', () {
        expect(
          const ReviewRevocationCodeRestartFlow(),
          const ReviewRevocationCodeRestartFlow(),
        );
      });
    });
  });
}
