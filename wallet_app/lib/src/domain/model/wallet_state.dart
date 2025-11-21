import 'package:equatable/equatable.dart';

sealed class WalletState extends Equatable {
  const WalletState();

  @override
  List<Object?> get props => [];
}

class WalletStateReady extends WalletState {
  const WalletStateReady();
}

class WalletStateEmpty extends WalletState {
  const WalletStateEmpty();
}

class WalletStateLocked extends WalletState {
  final WalletState substate;

  const WalletStateLocked(this.substate);

  @override
  List<Object?> get props => [...super.props, substate];
}

class WalletStateTransferPossible extends WalletState {
  const WalletStateTransferPossible();
}

class WalletStateTransferring extends WalletState {
  final TransferRole role;

  const WalletStateTransferring(this.role);

  @override
  List<Object?> get props => [...super.props, role];
}

class WalletStateRegistration extends WalletState {
  const WalletStateRegistration();
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

  @override
  List<Object?> get props => [...super.props, reason];
}

enum TransferRole { source, target }

enum WalletBlockedReason { requiresAppUpdate, blockedByWalletProvider }
