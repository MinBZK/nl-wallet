sealed class WalletState {
  const WalletState();
}

class WalletStateReady extends WalletState {
  const WalletStateReady();
}

class WalletStateTransferPossible extends WalletState {
  const WalletStateTransferPossible();
}

class WalletStateTransferring extends WalletState {
  final TransferRole role;

  const WalletStateTransferring(this.role);
}

class WalletStateRegistration extends WalletState {
  final bool hasConfiguredPin;

  const WalletStateRegistration({required this.hasConfiguredPin});
}

class WalletStateDisclosure extends WalletState {
  const WalletStateDisclosure();
}

class WalletStateIssuance extends WalletState {
  const WalletStateIssuance();
}

class WalletStatePinChange extends WalletState {
  const WalletStatePinChange();
}

class WalletStatePinRecovery extends WalletState {
  const WalletStatePinRecovery();
}

class WalletStateWalletBlocked extends WalletState {
  final WalletBlockedReason reason;

  const WalletStateWalletBlocked(this.reason);
}

enum TransferRole { source, target }

enum WalletBlockedReason { requiresAppUpdate, blockedByWalletProvider }
