import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/wallet_card.dart';
import '../../../../domain/usecase/card/lock_wallet_usecase.dart';
import '../../../../domain/usecase/card/observe_wallet_cards_usecase.dart';
import '../../../../wallet_constants.dart';

part 'card_overview_event.dart';
part 'card_overview_state.dart';

class CardOverviewBloc extends Bloc<CardOverviewEvent, CardOverviewState> {
  final LockWalletUseCase lockWalletUseCase;
  final ObserveWalletCardsUseCase observeWalletCardsUseCase;

  CardOverviewBloc(this.lockWalletUseCase, this.observeWalletCardsUseCase) : super(const CardOverviewInitial()) {
    on<CardOverviewLoadTriggered>(_onCardOverviewLoadTriggered);
    on<CardOverviewLockWalletPressed>(_onCardOverviewLockWalletPressed);

    //Immediately start loading when bloc is created.
    add(CardOverviewLoadTriggered());
  }

  void _onCardOverviewLoadTriggered(event, emit) async {
    emit(const CardOverviewLoadInProgress());
    await Future.delayed(kDefaultMockDelay);
    emit.forEach(
      observeWalletCardsUseCase.invoke(),
      onData: (cards) => CardOverviewLoadSuccess(cards),
      onError: (ex, stack) {
        //Note: when providing onError like this the subscription is not cancelled on errors
        Fimber.e('Failed to observe cards', ex: ex, stacktrace: stack);
        return const CardOverviewLoadFailure();
      },
    );
  }

  void _onCardOverviewLockWalletPressed(event, emit) async {
    lockWalletUseCase.lock();
  }
}
