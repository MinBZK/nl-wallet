import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/transfer/wallet_transfer_status.dart';
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
      when(mockTransferRepository.getWalletTransferState()).thenAnswer((_) async => WalletTransferStatus.success);

      // Act
      final stream = useCase.invoke();

      // Assert
      await expectLater(
        stream,
        emitsInOrder([
          WalletTransferStatus.success,
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
        return first ? WalletTransferStatus.readyForDownload : WalletTransferStatus.success;
      });

      // Act
      final stream = useCase.invoke();

      // Assert
      await expectLater(
        stream,
        emitsInOrder([
          WalletTransferStatus.readyForDownload,
          WalletTransferStatus.success,
          emitsDone,
        ]),
      );
      verify(mockTransferRepository.getWalletTransferState()).called(2);
    });

    test('should emit cancelled and complete when repository returns cancelled immediately', () async {
      // Arrange
      when(mockTransferRepository.getWalletTransferState()).thenAnswer((_) async => WalletTransferStatus.cancelled);

      // Act
      final stream = useCase.invoke();

      // Assert
      await expectLater(
        stream,
        emitsInOrder([
          WalletTransferStatus.cancelled,
          emitsDone,
        ]),
      );
      verify(mockTransferRepository.getWalletTransferState()).called(1);
    });

    test('should emit error and complete when repository returns error immediately', () async {
      // Arrange
      when(mockTransferRepository.getWalletTransferState()).thenAnswer((_) async => WalletTransferStatus.error);

      // Act
      final stream = useCase.invoke();

      // Assert
      await expectLater(
        stream,
        emitsInOrder([
          WalletTransferStatus.error,
          emitsDone,
        ]),
      );
      verify(mockTransferRepository.getWalletTransferState()).called(1);
    });
  });
}
