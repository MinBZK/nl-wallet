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
  final bool showRenewOption;

  const CardDetailLoadSuccess(this.detail, {this.showRenewOption = false});

  @override
  List<Object> get props => [detail, showRenewOption];
}

class CardDetailLoadFailure extends CardDetailState {
  final String attestationId;

  const CardDetailLoadFailure(this.attestationId);

  @override
  List<Object> get props => [attestationId];
}
