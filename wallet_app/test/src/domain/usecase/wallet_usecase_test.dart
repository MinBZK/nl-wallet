import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/wallet_usecase.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

void main() {
  setUp(() {});

  test('Non Core-/ApplicationErrors are mapped into a GenericError', () async {
    final sourceException = Exception('Non Core-/ApplicationError');
    final usecase = TestUseCase();
    final result = await usecase.tryCatch(
      () {
        return throw sourceException;
      },
      '[errorDescription]',
    );
    expect(result.error, isA<GenericError>());
    expect(result.error?.sourceError, sourceException);
  });

  group('handleAppError tests', () {
    test('ApplicationError is forwarded as is', () async {
      final sourceError = Exception('sourceError');
      final error = ExternalScannerError(sourceError: sourceError);
      final errorStream = Stream.error(error).handleAppError('[errorDescription]');

      expect(
        errorStream,
        emitsError(
          isA<ExternalScannerError>().having(
            (error) => error.sourceError,
            'sourceError matches',
            sourceError,
          ),
        ),
      );
    });

    test('CoreError is mapped and forwarded', () async {
      final sourceException = const CoreHardwareKeyUnsupportedError('test');
      final errorStream = Stream.error(sourceException).handleAppError('[errorDescription]');
      expect(errorStream, emitsError(isA<HardwareUnsupportedError>()));
    });

    test('Normal exception is wrapped in GenericError', () async {
      final sourceException = Exception('Non Core-/ApplicationError');
      final errorStream = Stream.error(sourceException).handleAppError('[errorDescription]');
      expect(
        errorStream,
        emitsError(
          isA<GenericError>().having(
            (error) => error.sourceError,
            'sourceError matches',
            sourceException,
          ),
        ),
      );
    });
  });
}

class TestUseCase extends WalletUseCase {
  Future<bool> invoke() async => true;
}
