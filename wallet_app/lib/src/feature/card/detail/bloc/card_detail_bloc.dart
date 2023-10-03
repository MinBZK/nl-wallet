import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/wallet_card_detail.dart';
import '../../../../domain/usecase/card/observe_wallet_card_detail_usecase.dart';

part 'card_detail_event.dart';
part 'card_detail_state.dart';

class CardDetailBloc extends Bloc<CardDetailEvent, CardDetailState> {
  final ObserveWalletCardDetailUseCase observeWalletCardDetailUseCase;

  CardDetailBloc(this.observeWalletCardDetailUseCase) : super(CardDetailInitial()) {
    on<CardDetailLoadTriggered>(_onCardSummaryLoadTriggered);
  }

  void _onCardSummaryLoadTriggered(CardDetailLoadTriggered event, emit) async {
    if (state is! CardDetailLoadSuccess) emit(const CardDetailLoadInProgress());
    await emit.forEach(
      observeWalletCardDetailUseCase.invoke(event.cardId),
      onData: (cardDetail) => CardDetailLoadSuccess(cardDetail),
      onError: (ex, stack) {
        //Note: when providing onError like this the subscription is not cancelled on errors
        Fimber.e('Failed to observe card detail', ex: ex, stacktrace: stack);
        return CardDetailLoadFailure(event.cardId);
      },
    );
  }
}
