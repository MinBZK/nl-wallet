import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/transfer/core/core_transfer_repository.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/model/transfer/transfer_session_state.dart';
import 'package:wallet_core/core.dart' as core;

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
        when(mockWalletCore.pairWalletTransfer(testUri)).thenAnswer((_) async {});

        await transferRepository.pairWalletTransfer(testUri);

        verify(mockWalletCore.pairWalletTransfer(testUri)).called(1);
      });
    });

    group('prepareTransferWallet', () {
      test('should call walletCore.prepareTransferWallet and return its result', () async {
        const pin = '1234';
        const expectedResult = core.WalletInstructionResult_Ok();
        when(mockWalletCore.confirmWalletTransfer(pin)).thenAnswer((_) async => expectedResult);

        final result = await transferRepository.confirmWalletTransfer(pin);

        expect(result, expectedResult);
        verify(mockWalletCore.confirmWalletTransfer(pin)).called(1);
      });
    });

    group('transferWallet', () {
      test('should call through to walletCore.transferWallet', () async {
        const expectedResult = core.WalletInstructionResult_Ok();
        when(mockWalletCore.transferWallet()).thenAnswer((_) async => expectedResult);

        await transferRepository.transferWallet();
        verify(mockWalletCore.transferWallet()).called(1);
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
        when(mockWalletCore.getWalletTransferState()).thenAnswer((_) async => core.TransferSessionState.Created);
        final result = await transferRepository.getWalletTransferState();
        expect(result, TransferSessionState.created);
      });

      test('should return WalletTransferStatus.waitingForApproval when walletCore returns Paired', () async {
        when(mockWalletCore.getWalletTransferState()).thenAnswer((_) async => core.TransferSessionState.Paired);
        final result = await transferRepository.getWalletTransferState();
        expect(result, TransferSessionState.paired);
      });

      test('should return WalletTransferStatus.transferring when walletCore returns Uploaded', () async {
        when(mockWalletCore.getWalletTransferState()).thenAnswer((_) async => core.TransferSessionState.Uploaded);
        final result = await transferRepository.getWalletTransferState();
        expect(result, TransferSessionState.uploaded);
      });

      test('should return WalletTransferStatus.transferring when walletCore returns Uploaded', () async {
        when(mockWalletCore.getWalletTransferState()).thenAnswer((_) async => core.TransferSessionState.Confirmed);
        final result = await transferRepository.getWalletTransferState();
        expect(result, TransferSessionState.confirmed);
      });

      test('should return WalletTransferStatus.success when walletCore returns Success', () async {
        when(mockWalletCore.getWalletTransferState()).thenAnswer((_) async => core.TransferSessionState.Success);
        final result = await transferRepository.getWalletTransferState();
        expect(result, TransferSessionState.success);
      });

      test('should return WalletTransferStatus.cancelled when walletCore returns Cancelled', () async {
        when(mockWalletCore.getWalletTransferState()).thenAnswer((_) async => core.TransferSessionState.Canceled);
        final result = await transferRepository.getWalletTransferState();
        expect(result, TransferSessionState.cancelled);
      });

      test('should return WalletTransferStatus.error when walletCore returns Error', () async {
        when(mockWalletCore.getWalletTransferState()).thenAnswer((_) async => core.TransferSessionState.Error);
        final result = await transferRepository.getWalletTransferState();
        expect(result, TransferSessionState.error);
      });
    });
  });
}
