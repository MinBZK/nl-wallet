import 'package:fimber/fimber.dart';

import '../../../../data/repository/disclosure/disclosure_repository.dart';
import '../../../../data/repository/issuance/issuance_repository.dart';
import '../../../../data/repository/pin/pin_repository.dart';
import '../../../../data/repository/transfer/transfer_repository.dart';
import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../../util/extension/wallet_state_extension.dart';
import '../../../model/result/result.dart';
import '../../../model/wallet_state.dart';
import '../move_to_ready_state_usecase.dart';

class MoveToReadyStateUseCaseImpl extends MoveToReadyStateUseCase {
  final WalletRepository _walletRepository;
  final IssuanceRepository _issuanceRepository;
  final DisclosureRepository _disclosureRepository;
  final PinRepository _pinRepository;
  final TransferRepository _transferRepository;

  MoveToReadyStateUseCaseImpl(
    this._walletRepository,
    this._issuanceRepository,
    this._disclosureRepository,
    this._pinRepository,
    this._transferRepository,
  );

  @override
  Future<Result<bool>> invoke() async {
    return tryCatch(() async {
      final state = await _walletRepository.getWalletState();
      switch (state) {
        case WalletStateReady():
          return true;
        case WalletStateUnregistered():
        case WalletStateEmpty():
        case WalletStateBlocked():
        case WalletStateInPinChangeFlow():
          Fimber.d("Can't move to Ready state from $state");
          return false;
        case WalletStateLocked():
          Fimber.d('Wallet in locked state, not altering internal state');
          return state.unlockedState is WalletStateReady;
        case WalletStateTransferPossible():
          throw 'Destination transfer states should be explicitly cancelled: $state';
        case WalletStateTransferring(:final role):
          switch (role) {
            case TransferRole.source:
              await _transferRepository.cancelWalletTransfer();
            case TransferRole.destination:
              throw 'Destination transfer states should be explicitly cancelled: $state';
          }
        case WalletStateInDisclosureFlow():
          await _disclosureRepository.cancelDisclosure();
        case WalletStateInIssuanceFlow():
          await _issuanceRepository.cancelIssuance();
        case WalletStateInPinRecoveryFlow():
          await _pinRepository.cancelPinRecovery();
      }
      // Make sure we fetch a fresh state as it might have been altered above
      return (await _walletRepository.getWalletState()) is WalletStateReady;
    }, 'Failed to move to ready state');
  }
}
