import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/wallet_state.dart';
import 'package:wallet/src/domain/usecase/wallet/impl/move_to_ready_state_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/wallet/move_to_ready_state_usecase.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MoveToReadyStateUseCase usecase;
  late MockWalletRepository mockWalletRepository;
  late MockIssuanceRepository mockIssuanceRepository;
  late MockDisclosureRepository mockDisclosureRepository;
  late MockPinRepository mockPinRepository;

  setUp(() {
    mockWalletRepository = MockWalletRepository();
    mockIssuanceRepository = MockIssuanceRepository();
    mockDisclosureRepository = MockDisclosureRepository();
    mockPinRepository = MockPinRepository();

    usecase = MoveToReadyStateUseCaseImpl(
      mockWalletRepository,
      mockIssuanceRepository,
      mockDisclosureRepository,
      mockPinRepository,
    );
  });

  group('Given wallet is in a state that cannot be moved from', () {
    test('when in WalletStateReady, should return true', () async {
      when(mockWalletRepository.getWalletState()).thenAnswer((_) async => const WalletStateReady());
      final result = await usecase.invoke();
      expect(result.value, isTrue);
    });

    for (final state in [
      const WalletStateUnregistered(),
      const WalletStateEmpty(),
      const WalletStateBlocked(BlockedReason.blockedByWalletProvider, canRegisterNewAccount: false),
      const WalletStateInPinChangeFlow(),
    ]) {
      test('when in ${state.runtimeType}, should return false', () async {
        when(mockWalletRepository.getWalletState()).thenAnswer((_) async => state);
        final result = await usecase.invoke();
        expect(result.value, isFalse);
      });
    }

    test('when in WalletStateLocked and substate is ready, should return true', () async {
      when(mockWalletRepository.getWalletState()).thenAnswer((_) async => const WalletStateLocked(WalletStateReady()));
      final result = await usecase.invoke();
      expect(result.value, isTrue);
    });

    test('when in WalletStateLocked and substate is not ready, should return false', () async {
      when(mockWalletRepository.getWalletState()).thenAnswer((_) async => const WalletStateLocked(WalletStateEmpty()));
      final result = await usecase.invoke();
      expect(result.value, isFalse);
    });
  });

  group('Given wallet is in a state that should throw an error', () {
    for (final state in [
      const WalletStateTransferPossible(),
      const WalletStateTransferring(TransferRole.source),
    ]) {
      test('when in ${state.runtimeType}, should throw exception', () async {
        when(mockWalletRepository.getWalletState()).thenAnswer((_) async => state);
        final result = await usecase.invoke();
        expect(result.hasError, isTrue);
      });
    }
  });

  group('Given wallet is in a flow that needs to be cancelled', () {
    test('when in WalletStateInDisclosureFlow, should cancel disclosure', () async {
      when(mockWalletRepository.getWalletState()).thenAnswer((_) async => const WalletStateInDisclosureFlow());
      await usecase.invoke();
      verify(mockDisclosureRepository.cancelDisclosure()).called(1);
    });

    test('when in WalletStateInIssuanceFlow, should cancel issuance', () async {
      when(
        mockWalletRepository.getWalletState(),
      ).thenAnswer((_) async => const WalletStateInIssuanceFlow());
      await usecase.invoke();
      verify(mockIssuanceRepository.cancelIssuance()).called(1);
    });

    test('when in WalletStateInPinRecoveryFlow, should cancel pin recovery', () async {
      when(
        mockWalletRepository.getWalletState(),
      ).thenAnswer(
        (_) async => const WalletStateInPinRecoveryFlow(),
      );
      await usecase.invoke();
      verify(mockPinRepository.cancelPinRecovery()).called(1);
    });
  });
}
