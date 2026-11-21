import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/wallet_usecase.dart';
import 'package:wallet/src/util/extension/core_error_extension.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../mocks/wallet_mocks.mocks.dart';

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

    test('Stream closes after the first error', () async {
      final source = StreamController<int>();
      final stream = source.stream.handleAppError('[errorDescription]');
      source
        ..add(1)
        ..addError(Exception('error'))
        ..add(2);
      await expectLater(stream, emitsInOrder([1, emitsError(isA<GenericError>()), emitsDone]));
      expect(source.hasListener, isFalse);
    });

    test('Cancelling the subscription cancels the source right away', () async {
      final source = StreamController<int>();
      final subscription = source.stream.handleAppError('[errorDescription]').listen(null);
      await pumpEventQueue();
      expect(source.hasListener, isTrue);

      unawaited(subscription.cancel());
      expect(source.hasListener, isFalse);
    });

    test('Pausing the subscription pauses the source', () async {
      final source = StreamController<int>();
      final subscription = source.stream.handleAppError('[errorDescription]').listen(null);
      await pumpEventQueue();

      subscription.pause();
      expect(source.isPaused, isTrue);
      subscription.resume();
      expect(source.isPaused, isFalse);
      await subscription.cancel();
    });

    test('Source errors after cancelling do not surface through the cancel future', () async {
      final source = StreamController<int>();
      final subscription = source.stream.handleAppError('[errorDescription]').listen(null);
      await pumpEventQueue();

      final cancelled = subscription.cancel();
      source.addError(const CoreStateError('locked'));
      await expectLater(cancelled, completes);
    });

    test('A failing cancel of the source still emits the error and closes the stream', () async {
      final source = StreamController<int>(onCancel: () => Future<void>.error(Exception('cancel failed')));
      final stream = source.stream.handleAppError('[errorDescription]');
      source.addError(Exception('error'));
      await expectLater(stream, emitsInOrder([emitsError(isA<GenericError>()), emitsDone]));
    });

    test('A failing error mapping still emits a GenericError and closes the stream', () async {
      final previousNetworkRepository = CoreErrorExtension.networkRepository;
      addTearDown(() => CoreErrorExtension.networkRepository = previousNetworkRepository);
      final networkRepository = MockNetworkRepository();
      when(networkRepository.hasInternet()).thenThrow(Exception('connectivity check failed'));
      CoreErrorExtension.networkRepository = networkRepository;

      const sourceError = CoreNetworkError('offline');
      final stream = Stream<int>.error(sourceError).handleAppError('[errorDescription]');
      await expectLater(
        stream,
        emitsInOrder([
          emitsError(isA<GenericError>().having((error) => error.sourceError, 'sourceError', sourceError)),
          emitsDone,
        ]),
      );
    });
  });
}

class TestUseCase extends WalletUseCase {
  Future<bool> invoke() async => true;
}
