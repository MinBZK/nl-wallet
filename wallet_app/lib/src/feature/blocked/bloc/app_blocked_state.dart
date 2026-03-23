part of 'app_blocked_bloc.dart';

/// Base class for all states of the App Blocked feature.
sealed class AppBlockedState extends Equatable {
  const AppBlockedState();

  @override
  List<Object?> get props => [];
}

/// Initial state while the reason for the blockage is being determined.
class AppBlockedInitial extends AppBlockedState {}

/// State emitted when the app fails to determine the reason for the blockage.
class AppBlockedError extends AppBlockedState {
  const AppBlockedError();
}

/// Represents the state where this instance of the wallet is blocked by the administrative side.
///
/// This state corresponds to [WalletStateBlocked].
/// [canRegisterNewAccount] is false when the user is not allowed to re-register (permanent block).
class AppBlockedByAdmin extends AppBlockedState {
  final WalletStateBlocked walletState;

  /// Whether the user is allowed to register a new account after this block.
  bool get canRegisterNewAccount => walletState.canRegisterNewAccount;

  const AppBlockedByAdmin(this.walletState);

  @override
  List<Object?> get props => [...super.props, walletState];
}

/// Represents the state where the wallet solution is compromised and thus cannot be used.
class AppBlockedSolutionCompromised extends AppBlockedState {
  const AppBlockedSolutionCompromised();
}

/// Represents the state where the user intentionally revoked this wallet instance, by using the revocation code.
class AppBlockedByUser extends AppBlockedState {
  const AppBlockedByUser();
}
