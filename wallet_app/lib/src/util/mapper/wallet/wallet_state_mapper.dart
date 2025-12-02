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
      core.WalletState_Transferring() => WalletStateTransferring(switch (input.role) {
        core.TransferRole.Source => .source,
        core.TransferRole.Destination => .target,
      }),
      core.WalletState_TransferPossible() => const WalletStateTransferPossible(),
      core.WalletState_Unregistered() => const WalletStateRegistration(),
      core.WalletState_InDisclosureFlow() => const WalletStateDisclosure(),
      core.WalletState_InIssuanceFlow() => const WalletStateIssuance(),
      core.WalletState_InPinChangeFlow() => const WalletStatePinChange(),
      core.WalletState_InPinRecoveryFlow() => const WalletStatePinRecovery(),
      core.WalletState_Blocked() => WalletStateWalletBlocked(switch (input.reason) {
        core.BlockedReason.RequiresAppUpdate => .requiresAppUpdate,
        core.BlockedReason.BlockedByWalletProvider => .blockedByWalletProvider,
      }),
      core.WalletState_Empty() => const WalletStateEmpty(),
    };
  }
}
