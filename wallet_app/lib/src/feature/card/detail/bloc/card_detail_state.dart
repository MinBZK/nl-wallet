part of 'card_detail_bloc.dart';

sealed class CardDetailState extends Equatable {
  const CardDetailState();
}

class CardDetailInitial extends CardDetailState {
  @override
  List<Object> get props => [];
}

class CardDetailLoadInProgress extends CardDetailState {
  final WalletCard? card;

  const CardDetailLoadInProgress({this.card});

  @override
  List<Object?> get props => [card];
}

class CardDetailLoadSuccess extends CardDetailState {
  final WalletCardDetail detail;

  const CardDetailLoadSuccess(this.detail);

  @override
  List<Object> get props => [detail];
}

class CardDetailLoadFailure extends CardDetailState {
  final String cardId;

  const CardDetailLoadFailure(this.cardId);

  @override
  List<Object> get props => [cardId];
}
