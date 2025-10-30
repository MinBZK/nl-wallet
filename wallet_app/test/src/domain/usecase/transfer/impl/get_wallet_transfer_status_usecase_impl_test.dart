import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/transfer/transfer_session_state.dart';
import 'package:wallet/src/domain/usecase/transfer/impl/get_wallet_transfer_status_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late GetWalletTransferStatusUseCaseImpl useCase;
  late MockTransferRepository mockTransferRepository;

  setUp(() {
    mockTransferRepository = MockTransferRepository();
    useCase = GetWalletTransferStatusUseCaseImpl(mockTransferRepository);
  });

  group('GetWalletTransferStatusUseCaseImpl', () {
    test('should emit success and complete when repository returns success immediately', () async {
      // Arrange
      when(mockTransferRepository.getWalletTransferState()).thenAnswer((_) async => TransferSessionState.success);

      // Act
      final stream = useCase.invoke();

      // Assert
      await expectLater(
        stream,
        emitsInOrder([
          TransferSessionState.success,
          emitsDone,
        ]),
      );
      verify(mockTransferRepository.getWalletTransferState()).called(1);
    });

    test('should emit processing then success and complete', () async {
      bool firstCall = true;
      // Arrange
      when(mockTransferRepository.getWalletTransferState()).thenAnswer((_) async {
        final first = firstCall;
        firstCall = false;
        return first ? TransferSessionState.uploaded : TransferSessionState.success;
      });

      // Act
      final stream = useCase.invoke();

      // Assert
      await expectLater(
        stream,
        emitsInOrder([
          TransferSessionState.uploaded,
          TransferSessionState.success,
          emitsDone,
        ]),
      );
      verify(mockTransferRepository.getWalletTransferState()).called(2);
    });

    test('should emit cancelled and complete when repository returns cancelled immediately', () async {
      // Arrange
      when(mockTransferRepository.getWalletTransferState()).thenAnswer((_) async => TransferSessionState.cancelled);

      // Act
      final stream = useCase.invoke();

      // Assert
      await expectLater(
        stream,
        emitsInOrder([
          TransferSessionState.cancelled,
          emitsDone,
        ]),
      );
      verify(mockTransferRepository.getWalletTransferState()).called(1);
    });

    test('should emit error and complete when repository returns error immediately', () async {
      // Arrange
      when(mockTransferRepository.getWalletTransferState()).thenAnswer((_) async => TransferSessionState.error);

      // Act
      final stream = useCase.invoke();

      // Assert
      await expectLater(
        stream,
        emitsInOrder([
          TransferSessionState.error,
          emitsDone,
        ]),
      );
      verify(mockTransferRepository.getWalletTransferState()).called(1);
    });
  });
}
