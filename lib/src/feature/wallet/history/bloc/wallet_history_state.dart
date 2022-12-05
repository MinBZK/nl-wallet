part of 'wallet_history_bloc.dart';

abstract class WalletHistoryState extends Equatable {
  const WalletHistoryState();
}

class WalletHistoryInitial extends WalletHistoryState {
  @override
  List<Object> get props => [];
}

class WalletHistoryLoadInProgress extends WalletHistoryState {
  const WalletHistoryLoadInProgress();

  @override
  List<Object?> get props => [];
}

class WalletHistoryLoadSuccess extends WalletHistoryState {
  final List<TimelineAttribute> attributes;

  const WalletHistoryLoadSuccess(this.attributes);

  @override
  List<Object> get props => [attributes];
}

class WalletHistoryLoadFailure extends WalletHistoryState {
  const WalletHistoryLoadFailure();

  @override
  List<Object> get props => [];
}
