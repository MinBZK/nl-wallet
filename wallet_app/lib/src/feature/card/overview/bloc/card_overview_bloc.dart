import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/wallet_card.dart';
import '../../../../domain/usecase/card/observe_wallet_cards_usecase.dart';
import '../../../../wallet_constants.dart';

part 'card_overview_event.dart';
part 'card_overview_state.dart';

class CardOverviewBloc extends Bloc<CardOverviewEvent, CardOverviewState> {
  final ObserveWalletCardsUseCase observeWalletCardsUseCase;

  CardOverviewBloc(this.observeWalletCardsUseCase, List<WalletCard>? preloadedCards)
      : super(preloadedCards == null ? const CardOverviewInitial() : CardOverviewLoadSuccess(preloadedCards)) {
    on<CardOverviewLoadTriggered>(_onCardOverviewLoadTriggered);

    //Immediately start loading when bloc is created.
    add(const CardOverviewLoadTriggered());
  }

  void _onCardOverviewLoadTriggered(CardOverviewLoadTriggered event, emit) async {
    if (state is! CardOverviewLoadSuccess || event.forceRefresh) emit(const CardOverviewLoadInProgress());
    await Future.delayed(kDefaultMockDelay);
    await emit.forEach(
      observeWalletCardsUseCase.invoke(),
      onData: (cards) => CardOverviewLoadSuccess(cards),
      onError: (ex, stack) {
        //Note: when providing onError like this the subscription is not cancelled on errors
        Fimber.e('Failed to observe cards', ex: ex, stacktrace: stack);
        return const CardOverviewLoadFailure();
      },
    );
  }
}
