import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/model/transfer/wallet_transfer_status.dart';
import 'package:wallet/src/feature/wallet_transfer_source/bloc/wallet_transfer_source_bloc.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  final MockAcknowledgeWalletTransferUseCase mockAcknowledgeWalletTransferUseCase =
      MockAcknowledgeWalletTransferUseCase();
  final MockGetWalletTransferStatusUseCase mockGetWalletTransferStatusUseCase = MockGetWalletTransferStatusUseCase();
  final MockCancelWalletTransferUseCase mockCancelWalletTransferUseCase = MockCancelWalletTransferUseCase();

  WalletTransferSourceBloc createBloc() => WalletTransferSourceBloc(
    mockAcknowledgeWalletTransferUseCase,
    mockGetWalletTransferStatusUseCase,
    mockCancelWalletTransferUseCase,
  );

  blocTest(
    'verify initial state',
    build: createBloc,
    verify: (bloc) => expect(bloc.state, const WalletTransferInitial()),
  );

  blocTest(
    'WalletTransferGenericError is emitted when transfer can not be acknowledged',
    build: createBloc,
    setUp: () => when(
      mockAcknowledgeWalletTransferUseCase.invoke(any),
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
      when(mockAcknowledgeWalletTransferUseCase.invoke(any)).thenAnswer((_) async => const Result.success(null));
      when(mockGetWalletTransferStatusUseCase.invoke()).thenAnswer(
        (_) => Stream.fromIterable([
          WalletTransferStatus.waitingForApprovalAndUpload,
          WalletTransferStatus.transferring,
          WalletTransferStatus.success,
        ]),
      );
    },
    act: (bloc) async {
      bloc.add(const WalletTransferAcknowledgeTransferEvent('https://example.org/transfer'));
      await Future.delayed(const Duration(milliseconds: 10));
      bloc.add(const WalletTransferAgreeEvent());
      await Future.delayed(const Duration(milliseconds: 10));
      bloc.add(const WalletTransferPinConfirmedEvent());
    },
    expect: () => [
      isA<WalletTransferLoading>(),
      isA<WalletTransferIntroduction>(),
      isA<WalletTransferConfirmPin>(),
      isA<WalletTransferTransferring>(),
      isA<WalletTransferSuccess>(),
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
      await Future.delayed(const Duration(milliseconds: 10));
      bloc.add(const WalletTransferAgreeEvent());
      await Future.delayed(const Duration(milliseconds: 10));
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
      when(mockAcknowledgeWalletTransferUseCase.invoke(any)).thenAnswer((_) async => const Result.success(null));
      when(mockGetWalletTransferStatusUseCase.invoke()).thenAnswer((_) => Stream.value(WalletTransferStatus.error));
    },
    act: (bloc) async {
      bloc.add(const WalletTransferAcknowledgeTransferEvent('https://example.org/transfer'));
      await Future.delayed(const Duration(milliseconds: 10));
      bloc.add(const WalletTransferAgreeEvent());
      await Future.delayed(const Duration(milliseconds: 10));
      bloc.add(const WalletTransferPinConfirmedEvent());
    },
    expect: () => [
      isA<WalletTransferLoading>(),
      isA<WalletTransferIntroduction>(),
      isA<WalletTransferConfirmPin>(),
      isA<WalletTransferTransferring>(),
      isA<WalletTransferFailed>(),
    ],
  );
}
