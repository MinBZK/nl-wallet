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

class WalletStateUnregistered extends WalletState {
  const WalletStateUnregistered();
}

class WalletStateInDisclosureFlow extends WalletState {
  const WalletStateInDisclosureFlow();
}

class WalletStateInIssuanceFlow extends WalletState {
  const WalletStateInIssuanceFlow();
}

class WalletStateInPinChangeFlow extends WalletState {
  const WalletStateInPinChangeFlow();
}

class WalletStateInPinRecoveryFlow extends WalletState {
  const WalletStateInPinRecoveryFlow();
}

class WalletStateBlocked extends WalletState {
  final BlockedReason reason;
  final bool canRegisterNewAccount;

  const WalletStateBlocked(this.reason, {required this.canRegisterNewAccount});

  @override
  List<Object?> get props => [...super.props, reason, canRegisterNewAccount];
}

enum TransferRole { source, target }

enum BlockedReason { requiresAppUpdate, blockedByWalletProvider }
