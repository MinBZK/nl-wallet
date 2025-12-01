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
        core.WalletTransferRole.Source => .source,
        core.WalletTransferRole.Destination => .target,
      }),
      core.WalletState_TransferPossible() => const WalletStateTransferPossible(),
      core.WalletState_Registration() => const WalletStateRegistration(),
      core.WalletState_Disclosure() => const WalletStateDisclosure(),
      core.WalletState_Issuance() => const WalletStateIssuance(),
      core.WalletState_PinChange() => const WalletStatePinChange(),
      core.WalletState_PinRecovery() => const WalletStatePinRecovery(),
      core.WalletState_WalletBlocked() => WalletStateWalletBlocked(switch (input.reason) {
        core.WalletBlockedReason.RequiresAppUpdate => .requiresAppUpdate,
        core.WalletBlockedReason.BlockedByWalletProvider => .blockedByWalletProvider,
      }),
      core.WalletState_Empty() => const WalletStateEmpty(),
    };
  }
}
