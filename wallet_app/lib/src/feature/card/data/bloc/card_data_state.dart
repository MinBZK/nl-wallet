part of 'card_data_bloc.dart';

sealed class CardDataState extends Equatable {
  const CardDataState();
}

class CardDataInitial extends CardDataState {
  @override
  List<Object> get props => [];
}

class CardDataLoadInProgress extends CardDataState {
  const CardDataLoadInProgress();

  @override
  List<Object?> get props => [];
}

class CardDataLoadSuccess extends CardDataState {
  final WalletCard card;

  const CardDataLoadSuccess(this.card);

  @override
  List<Object> get props => [card];
}

class CardDataLoadFailure extends CardDataState {
  const CardDataLoadFailure();

  @override
  List<Object> get props => [];
}
