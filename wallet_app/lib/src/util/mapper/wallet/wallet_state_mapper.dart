import 'package:wallet_core/core.dart' as core;

import '../../../domain/model/wallet_state.dart';
import '../mapper.dart';

class WalletStateMapper extends Mapper<core.WalletState, WalletState> {
  WalletStateMapper();

  @override
  WalletState map(core.WalletState input) {
    return switch (input) {
      core.WalletState_Ready() => const WalletStateReady(),
      core.WalletState_Locked() => WalletStateLocked(map(input.subState)),
      core.WalletState_Transferring() => _mapTransferring(input),
      core.WalletState_TransferPossible() => const WalletStateTransferPossible(),
      core.WalletState_Unregistered() => const WalletStateUnregistered(),
      core.WalletState_InDisclosureFlow() => const WalletStateInDisclosureFlow(),
      core.WalletState_InIssuanceFlow() => const WalletStateInIssuanceFlow(),
      core.WalletState_InPinChangeFlow() => const WalletStateInPinChangeFlow(),
      core.WalletState_InPinRecoveryFlow() => const WalletStateInPinRecoveryFlow(),
      core.WalletState_Blocked() => _mapBlocked(input),
      core.WalletState_Empty() => const WalletStateEmpty(),
    };
  }

  WalletStateTransferring _mapTransferring(core.WalletState_Transferring input) {
    final transferRole = switch (input.role) {
      core.TransferRole.Source => TransferRole.source,
      core.TransferRole.Destination => TransferRole.destination,
    };
    return WalletStateTransferring(transferRole);
  }

  WalletStateBlocked _mapBlocked(core.WalletState_Blocked input) {
    final blockedReason = switch (input.reason) {
      core.BlockedReason.RequiresAppUpdate => BlockedReason.requiresAppUpdate,
      core.BlockedReason.BlockedByWalletProvider => BlockedReason.blockedByWalletProvider,
      core.BlockedReason.WalletSolutionRevoked => BlockedReason.solutionRevoked,
    };
    return WalletStateBlocked(blockedReason, canRegisterNewAccount: input.canRegisterNewAccount);
  }
}
