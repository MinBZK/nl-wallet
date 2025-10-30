import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/model/transfer/transfer_session_state.dart';
import 'package:wallet/src/feature/wallet_transfer_source/bloc/wallet_transfer_source_bloc.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  late MockPairWalletTransferUseCase mockPairWalletTransferUseCase;
  late MockGetWalletTransferStatusUseCase mockGetWalletTransferStatusUseCase;
  late MockCancelWalletTransferUseCase mockCancelWalletTransferUseCase;
  late MockStartWalletTransferUseCase mockStartWalletTransferUseCase;
  late MockAutoLockService mockAutoLockService;

  setUp(() {
    mockPairWalletTransferUseCase = MockPairWalletTransferUseCase();
    mockGetWalletTransferStatusUseCase = MockGetWalletTransferStatusUseCase();
    mockCancelWalletTransferUseCase = MockCancelWalletTransferUseCase();
    mockStartWalletTransferUseCase = MockStartWalletTransferUseCase();
    mockAutoLockService = MockAutoLockService();
  });

  WalletTransferSourceBloc createBloc() {
    return WalletTransferSourceBloc(
      mockPairWalletTransferUseCase,
      mockGetWalletTransferStatusUseCase,
      mockCancelWalletTransferUseCase,
      mockStartWalletTransferUseCase,
      mockAutoLockService,
    );
  }

  blocTest(
    'verify initial state',
    build: createBloc,
    verify: (bloc) => expect(bloc.state, const WalletTransferInitial()),
  );

  blocTest(
    'WalletTransferGenericError is emitted when transfer can not be acknowledged',
    build: createBloc,
    setUp: () => when(
      mockPairWalletTransferUseCase.invoke(any),
    ).thenAnswer((_) async => const Result.error(GenericError('', sourceError: 'test'))),
    act: (bloc) => bloc.add(const WalletTransferAcknowledgeTransferEvent('https://example.org/transfer')),
    expect: () => [
      isA<WalletTransferLoading>(),
      isA<WalletTransferGenericError>(),
    ],
  );

  blocTest(
    'verify happy path',
    build: createBloc,
    setUp: () {
      when(mockPairWalletTransferUseCase.invoke(any)).thenAnswer((_) async => const Result.success(null));
      when(mockGetWalletTransferStatusUseCase.invoke()).thenAnswer(
        (_) => Stream.fromIterable([
          TransferSessionState.confirmed,
          TransferSessionState.uploaded,
          TransferSessionState.success,
        ]).delay(const Duration(milliseconds: 10)),
      );
    },
    act: (bloc) async {
      bloc.add(const WalletTransferAcknowledgeTransferEvent('https://example.org/transfer'));
      await Future.delayed(Duration.zero);
      bloc.add(const WalletTransferAgreeEvent());
      await Future.delayed(Duration.zero);
      bloc.add(const WalletTransferPinConfirmedEvent());
      // Wait for (mock) stream to emit
      await Future.delayed(const Duration(milliseconds: 15));
    },
    verify: (bloc) => verify(mockStartWalletTransferUseCase.invoke()).called(1),
    expect: () => [
      // Initial state
      isA<WalletTransferLoading>(),
      // State once [WalletTransferAcknowledgeTransferEvent] is processed
      isA<WalletTransferIntroduction>(),
      // State once [WalletTransferAgreeEvent] is processed
      isA<WalletTransferConfirmPin>(),
      // State once [WalletTransferPinConfirmedEvent] is processed
      isA<WalletTransferTransferring>(),
      // State after [readyForTransferConfirmed] is processed (which calls StartWalletTransferUseCase)
      isA<WalletTransferSuccess>(),
    ],
  );

  blocTest(
    'verify network error that occurs during transfer',
    build: createBloc,
    setUp: () {
      when(mockPairWalletTransferUseCase.invoke(any)).thenAnswer((_) async => const Result.success(null));
      when(mockGetWalletTransferStatusUseCase.invoke()).thenAnswer(
        (_) => Stream.fromIterable([
          TransferSessionState.confirmed,
        ]).delay(const Duration(milliseconds: 10)),
      );
      when(
        mockStartWalletTransferUseCase.invoke(),
      ).thenAnswer((_) async => const Result.error(NetworkError(hasInternet: false, sourceError: 'test')));
    },
    act: (bloc) async {
      bloc.add(const WalletTransferAcknowledgeTransferEvent('https://example.org/transfer'));
      await Future.delayed(Duration.zero);
      bloc.add(const WalletTransferAgreeEvent());
      await Future.delayed(Duration.zero);
      bloc.add(const WalletTransferPinConfirmedEvent());
      // Wait for (mock) stream to emit
      await Future.delayed(const Duration(milliseconds: 15));
    },
    verify: (bloc) => verify(mockStartWalletTransferUseCase.invoke()).called(1),
    expect: () => [
      // Initial state
      isA<WalletTransferLoading>(),
      // State once [WalletTransferAcknowledgeTransferEvent] is processed
      isA<WalletTransferIntroduction>(),
      // State once [WalletTransferAgreeEvent] is processed
      isA<WalletTransferConfirmPin>(),
      // State once [WalletTransferPinConfirmedEvent] is processed
      isA<WalletTransferTransferring>(),
      // State after [readyForTransferConfirmed] is processed (which calls StartWalletTransferUseCase)
      isA<WalletTransferNetworkError>(),
    ],
  );

  blocTest(
    'verify stop requested event',
    build: createBloc,
    setUp: () => when(mockCancelWalletTransferUseCase.invoke()).thenAnswer((_) async => const Result.success(null)),
    act: (bloc) => bloc.add(const WalletTransferStopRequestedEvent()),
    expect: () => [isA<WalletTransferStopped>()],
    verify: (_) => verify(mockCancelWalletTransferUseCase.invoke()).called(1),
  );

  blocTest(
    'verify back pressed event from confirm pin',
    build: createBloc,
    act: (bloc) async {
      bloc.add(const WalletTransferAcknowledgeTransferEvent('https://example.org/transfer'));
      await Future.delayed(Duration.zero);
      bloc.add(const WalletTransferAgreeEvent());
      await Future.delayed(Duration.zero);
      bloc.add(const WalletTransferBackPressedEvent());
    },
    expect: () => [
      isA<WalletTransferLoading>(),
      isA<WalletTransferIntroduction>(),
      isA<WalletTransferConfirmPin>(),
      isA<WalletTransferIntroduction>().having((it) => it.didGoBack, 'didGoBack is true', true),
    ],
  );

  blocTest(
    'verify transfer failed with generic error',
    build: createBloc,
    setUp: () {
      when(mockPairWalletTransferUseCase.invoke(any)).thenAnswer((_) async => const Result.success(null));
      when(
        mockGetWalletTransferStatusUseCase.invoke(),
      ).thenAnswer((_) => Stream.value(TransferSessionState.error).delay(const Duration(milliseconds: 10)));
    },
    act: (bloc) async {
      bloc.add(const WalletTransferAcknowledgeTransferEvent('https://example.org/transfer'));
      await Future.delayed(Duration.zero);
      bloc.add(const WalletTransferAgreeEvent());
      await Future.delayed(Duration.zero);
      bloc.add(const WalletTransferPinConfirmedEvent());
      // Wait for (mock) stream to emit
      await Future.delayed(const Duration(milliseconds: 20));
    },
    expect: () => [
      isA<WalletTransferLoading>(),
      isA<WalletTransferIntroduction>(),
      isA<WalletTransferConfirmPin>(),
      isA<WalletTransferTransferring>(),
      isA<WalletTransferFailed>(),
    ],
  );

  blocTest(
    'verify transfer can be cancelled by destination',
    build: createBloc,
    setUp: () {
      when(mockPairWalletTransferUseCase.invoke(any)).thenAnswer((_) async => const Result.success(null));
      when(
        mockGetWalletTransferStatusUseCase.invoke(),
      ).thenAnswer((_) => Stream.value(TransferSessionState.cancelled).delay(const Duration(milliseconds: 10)));
    },
    act: (bloc) async {
      bloc.add(const WalletTransferAcknowledgeTransferEvent('https://example.org/transfer'));
      // Wait for (mock) stream to emit
      await Future.delayed(const Duration(milliseconds: 20));
    },
    expect: () => [
      isA<WalletTransferLoading>(),
      isA<WalletTransferIntroduction>(),
      isA<WalletTransferStopped>(),
    ],
  );

  blocTest(
    'verify bloc emits WalletTransferNetworkError when the get status throws a NetworkError',
    build: createBloc,
    setUp: () {
      when(mockPairWalletTransferUseCase.invoke(any)).thenAnswer((_) async => const Result.success(null));
      when(
        mockGetWalletTransferStatusUseCase.invoke(),
      ).thenAnswer(
        (_) async* {
          await Future.delayed(const Duration(milliseconds: 10));
          throw const NetworkError(hasInternet: false, sourceError: 'network_error');
        },
      );
    },
    act: (bloc) async {
      bloc.add(const WalletTransferAcknowledgeTransferEvent('https://example.org/transfer'));
      await Future.delayed(Duration.zero);
      bloc.add(const WalletTransferAgreeEvent());
      await Future.delayed(Duration.zero);
      bloc.add(const WalletTransferPinConfirmedEvent());
      // Wait for (mock) stream to emit
      await Future.delayed(const Duration(milliseconds: 20));
    },
    expect: () => [
      isA<WalletTransferLoading>(),
      isA<WalletTransferIntroduction>(),
      isA<WalletTransferConfirmPin>(),
      isA<WalletTransferTransferring>(),
      isA<WalletTransferNetworkError>(),
    ],
  );

  blocTest(
    'verify autolock is re-enabled when bloc is closed',
    setUp: () => reset(mockAutoLockService),
    build: createBloc,
    verify: (bloc) {
      expect(bloc.isClosed, isTrue, reason: 'BLoC should (automatically) be closed');
      verify(mockAutoLockService.setAutoLock(enabled: true)).called(1);
    },
  );
}
