part of 'history_detail_bloc.dart';

sealed class HistoryDetailState extends Equatable {
  const HistoryDetailState();
}

class HistoryDetailInitial extends HistoryDetailState {
  @override
  List<Object> get props => [];
}

class HistoryDetailLoadInProgress extends HistoryDetailState {
  const HistoryDetailLoadInProgress();

  @override
  List<Object?> get props => [];
}

class HistoryDetailLoadSuccess extends HistoryDetailState {
  final WalletEvent event;

  const HistoryDetailLoadSuccess(this.event);

  WalletCard? cardById(String attestationId) {
    final event = this.event;
    switch (event) {
      case DisclosureEvent():
        return event.cards.firstWhereOrNull((card) => card.attestationId == attestationId);
      case IssuanceEvent():
        return event.card.takeIf((card) => card.attestationId == attestationId);
      case SignEvent():
        return null;
    }
  }

  @override
  List<Object> get props => [event];
}

class HistoryDetailLoadFailure extends HistoryDetailState {
  const HistoryDetailLoadFailure();

  @override
  List<Object> get props => [];
}
