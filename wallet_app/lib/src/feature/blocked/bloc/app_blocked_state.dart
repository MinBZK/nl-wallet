part of 'app_blocked_bloc.dart';

sealed class AppBlockedState extends Equatable {
  const AppBlockedState();
}

class AppBlockedInitial extends AppBlockedState {
  @override
  List<Object> get props => [];
}

class AppBlockedError extends AppBlockedState {
  const AppBlockedError();

  @override
  List<Object> get props => [];
}

class AppBlockedByAdmin extends AppBlockedState {
  final WalletStateBlocked walletState;

  bool get canRegisterNewAccount => walletState.canRegisterNewAccount;

  const AppBlockedByAdmin(this.walletState);

  @override
  List<Object> get props => [walletState];
}

class AppBlockedByUser extends AppBlockedState {
  const AppBlockedByUser();

  @override
  List<Object> get props => [];
}
