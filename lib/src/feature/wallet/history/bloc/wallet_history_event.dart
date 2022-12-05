part of 'wallet_history_bloc.dart';

abstract class WalletHistoryEvent extends Equatable {
  const WalletHistoryEvent();
}

class WalletHistoryLoadTriggered extends WalletHistoryEvent {
  const WalletHistoryLoadTriggered();

  @override
  List<Object?> get props => [];
}
