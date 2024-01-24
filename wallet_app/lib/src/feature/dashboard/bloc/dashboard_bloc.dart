import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/wallet_card.dart';
import '../../../domain/usecase/card/observe_wallet_cards_usecase.dart';
import '../../../wallet_constants.dart';

part 'dashboard_event.dart';
part 'dashboard_state.dart';

class DashboardBloc extends Bloc<DashboardEvent, DashboardState> {
  final ObserveWalletCardsUseCase observeWalletCardsUseCase;

  DashboardBloc(this.observeWalletCardsUseCase, List<WalletCard>? preloadedCards)
      : super(preloadedCards == null ? const DashboardStateInitial() : DashboardLoadSuccess(preloadedCards)) {
    on<DashboardLoadTriggered>(_onCardOverviewLoadTriggered);

    //Immediately start loading when bloc is created.
    add(const DashboardLoadTriggered());
  }

  void _onCardOverviewLoadTriggered(DashboardLoadTriggered event, emit) async {
    if (state is! DashboardLoadSuccess || event.forceRefresh) emit(const DashboardLoadInProgress());
    await Future.delayed(kDefaultMockDelay);
    await emit.forEach(
      observeWalletCardsUseCase.invoke(),
      onData: (cards) => DashboardLoadSuccess(cards),
      onError: (ex, stack) {
        //Note: when providing onError like this the subscription is not cancelled on errors
        Fimber.e('Failed to observe cards', ex: ex, stacktrace: stack);
        return const DashboardLoadFailure();
      },
    );
  }
}
