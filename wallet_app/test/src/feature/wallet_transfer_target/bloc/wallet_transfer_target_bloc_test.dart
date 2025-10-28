import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/model/transfer/wallet_transfer_status.dart';
import 'package:wallet/src/feature/wallet_transfer_target/bloc/wallet_transfer_target_bloc.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  late MockInitWalletTransferUseCase mockInitWalletTransferUseCase;
  late MockGetWalletTransferStatusUseCase mockGetWalletTransferStatusUseCase;
  late MockCancelWalletTransferUseCase mockCancelWalletTransferUseCase;
  late MockSkipWalletTransferUseCase mockSkipWalletTransferUseCase;
  late MockReceiveWalletTransferUseCase mockReceiveWalletTransferUseCase;
  late MockAutoLockService mockAutoLockService;

  setUp(() {
    mockInitWalletTransferUseCase = MockInitWalletTransferUseCase();
    mockGetWalletTransferStatusUseCase = MockGetWalletTransferStatusUseCase();
    mockCancelWalletTransferUseCase = MockCancelWalletTransferUseCase();
    mockSkipWalletTransferUseCase = MockSkipWalletTransferUseCase();
    mockReceiveWalletTransferUseCase = MockReceiveWalletTransferUseCase();
    mockAutoLockService = MockAutoLockService();
  });

  WalletTransferTargetBloc createBloc() => WalletTransferTargetBloc(
    mockInitWalletTransferUseCase,
    mockGetWalletTransferStatusUseCase,
    mockCancelWalletTransferUseCase,
    mockSkipWalletTransferUseCase,
    mockReceiveWalletTransferUseCase,
    mockAutoLockService,
  );

  blocTest<WalletTransferTargetBloc, WalletTransferTargetState>(
    'verify initial state',
    build: createBloc,
    verify: (bloc) => expect(bloc.state, const WalletTransferIntroduction()),
  );

  group('WalletTransferOptInEvent', () {
    const qrData = 'test_qr_data';

    blocTest<WalletTransferTargetBloc, WalletTransferTargetState>(
      'happy path',
      build: createBloc,
      setUp: () {
        when(mockInitWalletTransferUseCase.invoke()).thenAnswer((_) async => const Result.success(qrData));
        when(mockGetWalletTransferStatusUseCase.invoke()).thenAnswer(
          (_) => Stream.fromIterable([
            WalletTransferStatus.waitingForScan,
            WalletTransferStatus.waitingForApprovalAndUpload,
            WalletTransferStatus.readyForDownload,
            WalletTransferStatus.success,
          ]),
        );
      },
      act: (bloc) => bloc.add(const WalletTransferOptInEvent()),
      expect: () => [
        const WalletTransferLoadingQrData(),
        const WalletTransferAwaitingQrScan(qrData),
        const WalletTransferAwaitingConfirmation(),
        const WalletTransferTransferring(),
        const WalletTransferSuccess(),
      ],
    );

    blocTest<WalletTransferTargetBloc, WalletTransferTargetState>(
      'prepare transfer fails with GenericError',
      build: createBloc,
      setUp: () => when(
        mockInitWalletTransferUseCase.invoke(),
      ).thenAnswer((_) async => const Result.error(GenericError('prepare_fail', sourceError: 'prepare'))),
      act: (bloc) => bloc.add(const WalletTransferOptInEvent()),
      expect: () => [
        const WalletTransferLoadingQrData(),
        isA<WalletTransferGenericError>().having((e) => e.error, 'error', isA<GenericError>()),
      ],
    );

    blocTest<WalletTransferTargetBloc, WalletTransferTargetState>(
      'prepare transfer fails with NetworkError',
      build: createBloc,
      setUp: () => when(mockInitWalletTransferUseCase.invoke()).thenAnswer(
        (_) async => const Result.error(NetworkError(hasInternet: false, sourceError: 'prepare_fail_network')),
      ),
      act: (bloc) => bloc.add(const WalletTransferOptInEvent()),
      expect: () => [
        const WalletTransferLoadingQrData(),
        isA<WalletTransferNetworkError>().having((e) => e.hasInternet, 'error', isFalse),
      ],
    );

    blocTest<WalletTransferTargetBloc, WalletTransferTargetState>(
      'status stream emits WalletTransferStatus.error after scan',
      build: createBloc,
      setUp: () {
        when(mockInitWalletTransferUseCase.invoke()).thenAnswer((_) async => const Result.success(qrData));
        when(mockGetWalletTransferStatusUseCase.invoke()).thenAnswer(
          (_) => Stream.fromIterable([
            WalletTransferStatus.waitingForScan,
            WalletTransferStatus.error,
          ]),
        );
      },
      act: (bloc) => bloc.add(const WalletTransferOptInEvent()),
      expect: () => [
        const WalletTransferLoadingQrData(),
        const WalletTransferAwaitingQrScan(qrData),
        isA<WalletTransferFailed>(),
      ],
    );

    blocTest<WalletTransferTargetBloc, WalletTransferTargetState>(
      'status stream itself throws an exception after scan',
      build: createBloc,
      setUp: () {
        when(mockInitWalletTransferUseCase.invoke()).thenAnswer((_) async => const Result.success(qrData));
        when(
          mockGetWalletTransferStatusUseCase.invoke(),
        ).thenAnswer((_) => Stream.error(Exception('Stream failed')));
      },
      act: (bloc) => bloc.add(const WalletTransferOptInEvent()),
      expect: () => [
        const WalletTransferLoadingQrData(),
        const WalletTransferAwaitingQrScan(qrData),
        isA<WalletTransferGenericError>().having((e) => e.error.sourceError, 'sourceError', isA<Exception>()),
      ],
    );

    blocTest<WalletTransferTargetBloc, WalletTransferTargetState>(
      'when status stream throws NetworkError emit WalletTransferNetworkError',
      build: createBloc,
      setUp: () {
        when(mockInitWalletTransferUseCase.invoke()).thenAnswer((_) async => const Result.success(qrData));
        when(
          mockGetWalletTransferStatusUseCase.invoke(),
        ).thenAnswer(
          (_) => Stream.error(const NetworkError(hasInternet: false, sourceError: CoreNetworkError('network error'))),
        );
      },
      act: (bloc) => bloc.add(const WalletTransferOptInEvent()),
      expect: () => [
        const WalletTransferLoadingQrData(),
        const WalletTransferAwaitingQrScan(qrData),
        isA<WalletTransferNetworkError>(),
      ],
    );
  });

  blocTest<WalletTransferTargetBloc, WalletTransferTargetState>(
    'WalletTransferOptOutEvent calls skip use case and emits nothing',
    build: createBloc,
    act: (bloc) => bloc.add(const WalletTransferOptOutEvent()),
    expect: () => [],
    verify: (_) {
      verify(mockSkipWalletTransferUseCase.invoke()).called(1);
    },
  );

  group('WalletTransferStopRequestedEvent', () {
    const qrData = 'test_qr_data_for_stop';
    blocTest<WalletTransferTargetBloc, WalletTransferTargetState>(
      'from WalletTransferAwaitingQrScan state',
      build: createBloc,
      setUp: () {
        when(mockInitWalletTransferUseCase.invoke()).thenAnswer((_) async => const Result.success(qrData));
        // Let status stream emit waitingForScan to reach the desired state
        when(
          mockGetWalletTransferStatusUseCase.invoke(),
        ).thenAnswer((_) => Stream.value(WalletTransferStatus.waitingForScan).asBroadcastStream());
        when(mockCancelWalletTransferUseCase.invoke()).thenAnswer((_) async => const Result.success(null));
      },
      act: (bloc) async {
        bloc.add(const WalletTransferOptInEvent());
        await untilCalled(mockGetWalletTransferStatusUseCase.invoke()); // Ensure OptIn processing starts
        await Future.delayed(Duration.zero); // Allow stream to emit and bloc to process
        bloc.add(const WalletTransferStopRequestedEvent());
      },
      expect: () => [
        const WalletTransferLoadingQrData(),
        const WalletTransferAwaitingQrScan(qrData),
        const WalletTransferStopped(),
      ],
      verify: (_) {
        verify(mockCancelWalletTransferUseCase.invoke()).called(1);
      },
    );
  });

  blocTest<WalletTransferTargetBloc, WalletTransferTargetState>(
    'WalletTransferRestartEvent emits WalletTransferIntroduction with didGoBack true',
    build: createBloc,
    act: (bloc) => bloc.add(const WalletTransferRestartEvent()),
    expect: () => [const WalletTransferIntroduction(didGoBack: true)],
  );

  group('WalletTransferBackPressedEvent', () {
    const qrData = 'test_qr_data_for_back';
    blocTest<WalletTransferTargetBloc, WalletTransferTargetState>(
      'should navigate from WalletTransferAwaitingQrScan state (canGoBack is true)',
      build: createBloc,
      setUp: () {
        when(mockInitWalletTransferUseCase.invoke()).thenAnswer((_) async => const Result.success(qrData));
        when(
          mockGetWalletTransferStatusUseCase.invoke(),
        ).thenAnswer((_) => Stream.value(WalletTransferStatus.waitingForScan).asBroadcastStream());
      },
      act: (bloc) async {
        bloc.add(const WalletTransferOptInEvent());
        await untilCalled(mockGetWalletTransferStatusUseCase.invoke());
        await Future.delayed(Duration.zero);
        bloc.add(const WalletTransferBackPressedEvent());
      },
      expect: () => [
        const WalletTransferLoadingQrData(),
        const WalletTransferAwaitingQrScan(qrData),
        const WalletTransferIntroduction(didGoBack: true),
      ],
    );

    blocTest<WalletTransferTargetBloc, WalletTransferTargetState>(
      'should not navigate from WalletTransferTransferring state (canGoBack is false)',
      build: createBloc,
      setUp: () {
        when(mockInitWalletTransferUseCase.invoke()).thenAnswer((_) async => const Result.success(qrData));
        when(mockGetWalletTransferStatusUseCase.invoke()).thenAnswer(
          (_) => Stream.fromIterable([
            WalletTransferStatus.waitingForScan,
            WalletTransferStatus.readyForDownload,
          ]).asBroadcastStream(),
        );
        when(
          mockReceiveWalletTransferUseCase.invoke(),
        ).thenAnswer((_) async {
          await Future.delayed(const Duration(milliseconds: 5));
          return const Result.success(null);
        });
      },
      act: (bloc) async {
        bloc.add(const WalletTransferOptInEvent());
        await untilCalled(mockGetWalletTransferStatusUseCase.invoke());
        await Future.delayed(Duration.zero); // ensure transferring state is reached
        bloc.add(const WalletTransferBackPressedEvent());
        await Future.delayed(const Duration(milliseconds: 5)); // ensure success state is reached
      },
      expect: () => [
        // only initial opt-in states, no change from back press
        const WalletTransferLoadingQrData(),
        const WalletTransferAwaitingQrScan(qrData),
        const WalletTransferTransferring(),
        const WalletTransferSuccess(),
      ],
    );
  });

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
