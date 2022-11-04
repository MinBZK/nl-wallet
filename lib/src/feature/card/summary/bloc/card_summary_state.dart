part of 'card_summary_bloc.dart';

abstract class CardSummaryState extends Equatable {
  const CardSummaryState();
}

class CardSummaryInitial extends CardSummaryState {
  @override
  List<Object> get props => [];
}

class CardSummaryLoadInProgress extends CardSummaryState {
  const CardSummaryLoadInProgress();

  @override
  List<Object?> get props => [];
}

class CardSummaryLoadSuccess extends CardSummaryState {
  final WalletCardSummary summary;

  const CardSummaryLoadSuccess(this.summary);

  @override
  List<Object> get props => [summary];
}

class CardSummaryLoadFailure extends CardSummaryState {
  const CardSummaryLoadFailure();

  @override
  List<Object> get props => [];
}
