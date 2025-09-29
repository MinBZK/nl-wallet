import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/transfer/core/core_transfer_repository.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/model/transfer/wallet_transfer_status.dart';
import 'package:wallet_core/core.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late CoreTransferRepository transferRepository;
  late MockTypedWalletCore mockWalletCore;

  setUp(() {
    mockWalletCore = MockTypedWalletCore();
    transferRepository = CoreTransferRepository(mockWalletCore);
  });

  group('CoreTransferRepository', () {
    group('initWalletTransfer', () {
      test('should call walletCore.initWalletTransfer and return its result', () async {
        const expectedUri = 'test_uri';
        when(mockWalletCore.initWalletTransfer()).thenAnswer((_) async => expectedUri);

        final result = await transferRepository.initWalletTransfer();

        expect(result, expectedUri);
        verify(mockWalletCore.initWalletTransfer()).called(1);
      });
    });

    group('acknowledgeWalletTransfer', () {
      test('should call walletCore.acknowledgeWalletTransfer with the given uri', () async {
        const testUri = 'test_uri';
        when(mockWalletCore.acknowledgeWalletTransfer(testUri)).thenAnswer((_) async {});

        await transferRepository.acknowledgeWalletTransfer(testUri);

        verify(mockWalletCore.acknowledgeWalletTransfer(testUri)).called(1);
      });
    });

    group('transferWallet', () {
      test('should call walletCore.transferWallet and return its result', () async {
        const pin = '1234';
        const expectedResult = WalletInstructionResult_Ok();
        when(mockWalletCore.transferWallet(pin)).thenAnswer((_) async => expectedResult);

        final result = await transferRepository.transferWallet(pin);

        expect(result, expectedResult);
        verify(mockWalletCore.transferWallet(pin)).called(1);
      });
    });

    group('cancelWalletTransfer', () {
      test('should call walletCore.cancelWalletTransfer', () async {
        when(mockWalletCore.cancelWalletTransfer()).thenAnswer((_) async => const Result.success(null));

        await transferRepository.cancelWalletTransfer();

        verify(mockWalletCore.cancelWalletTransfer()).called(1);
      });
    });

    group('getWalletTransferState', () {
      // Test cases for each possible mapping from TransferSessionState to WalletTransferStatus
      test('should return WalletTransferStatus.waitingForScan when walletCore returns Created', () async {
        when(mockWalletCore.getWalletTransferState()).thenAnswer((_) async => TransferSessionState.Created);
        final result = await transferRepository.getWalletTransferState();
        expect(result, WalletTransferStatus.waitingForScan);
      });

      test('should return WalletTransferStatus.waitingForApproval when walletCore returns ReadyForTransfer', () async {
        when(mockWalletCore.getWalletTransferState()).thenAnswer((_) async => TransferSessionState.ReadyForTransfer);
        final result = await transferRepository.getWalletTransferState();
        expect(result, WalletTransferStatus.waitingForApprovalAndUpload);
      });

      test('should return WalletTransferStatus.transferring when walletCore returns ReadyForDownload', () async {
        when(mockWalletCore.getWalletTransferState()).thenAnswer((_) async => TransferSessionState.ReadyForDownload);
        final result = await transferRepository.getWalletTransferState();
        expect(result, WalletTransferStatus.transferring);
      });

      test('should return WalletTransferStatus.success when walletCore returns Success', () async {
        when(mockWalletCore.getWalletTransferState()).thenAnswer((_) async => TransferSessionState.Success);
        final result = await transferRepository.getWalletTransferState();
        expect(result, WalletTransferStatus.success);
      });

      test('should return WalletTransferStatus.cancelled when walletCore returns Cancelled', () async {
        when(mockWalletCore.getWalletTransferState()).thenAnswer((_) async => TransferSessionState.Cancelled);
        final result = await transferRepository.getWalletTransferState();
        expect(result, WalletTransferStatus.cancelled);
      });

      test('should return WalletTransferStatus.error when walletCore returns Error', () async {
        when(mockWalletCore.getWalletTransferState()).thenAnswer((_) async => TransferSessionState.Error);
        final result = await transferRepository.getWalletTransferState();
        expect(result, WalletTransferStatus.error);
      });
    });
  });
}
