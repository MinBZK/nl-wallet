import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/wallet_card.dart';
import '../../../../domain/usecase/card/get_wallet_cards_usecase.dart';
import '../../../../domain/usecase/card/lock_wallet_usecase.dart';

part 'card_overview_event.dart';
part 'card_overview_state.dart';

class CardOverviewBloc extends Bloc<CardOverviewEvent, CardOverviewState> {
  final LockWalletUseCase lockWalletUseCase;
  final GetWalletCardsUseCase getWalletCardsUseCase;

  CardOverviewBloc(this.lockWalletUseCase, this.getWalletCardsUseCase) : super(const CardOverviewInitial()) {
    on<CardOverviewLoadTriggered>(_onCardOverviewLoadTriggered);
    on<CardOverviewLockWalletPressed>(_onCardOverviewLockWalletPressed);

    //Immediately start loading when bloc is created.
    add(CardOverviewLoadTriggered());
  }

  void _onCardOverviewLoadTriggered(event, emit) async {
    try {
      List<WalletCard> cards = await getWalletCardsUseCase.getWalletCardsOrderedByIdAsc();
      emit(CardOverviewLoadSuccess(cards));
    } catch (error) {
      emit(const CardOverviewLoadFailure());
    }
  }

  void _onCardOverviewLockWalletPressed(event, emit) async {
    lockWalletUseCase.lock();
  }
}
