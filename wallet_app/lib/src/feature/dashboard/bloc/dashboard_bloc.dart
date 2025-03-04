import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:rxdart/rxdart.dart';

import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/model/event/wallet_event.dart';
import '../../../domain/usecase/card/observe_wallet_cards_usecase.dart';
import '../../../domain/usecase/history/observe_recent_history_usecase.dart';

part 'dashboard_event.dart';
part 'dashboard_state.dart';

class DashboardBloc extends Bloc<DashboardEvent, DashboardState> {
  final ObserveWalletCardsUseCase observeWalletCardsUseCase;
  final ObserveRecentHistoryUseCase observeRecentHistoryUseCase;

  DashboardBloc(
    this.observeWalletCardsUseCase,
    this.observeRecentHistoryUseCase,
    List<WalletCard>? preloadedCards,
  ) : super(preloadedCards == null ? const DashboardStateInitial() : DashboardLoadSuccess(cards: preloadedCards)) {
    on<DashboardLoadTriggered>(_onCardOverviewLoadTriggered, transformer: restartable());
  }

  Future<void> _onCardOverviewLoadTriggered(DashboardLoadTriggered event, Emitter<DashboardState> emit) async {
    if (state is! DashboardLoadSuccess || event.forceRefresh) emit(const DashboardLoadInProgress());
    await emit.forEach(
      CombineLatestStream.combine2(
        observeWalletCardsUseCase.invoke(),
        observeRecentHistoryUseCase.invoke(),
        (cards, history) => (cards, history),
      ),
      onData: (data) {
        final (cards, history) = data;
        return DashboardLoadSuccess(cards: cards, history: history);
      },
      onError: (ex, stack) {
        //Note: when providing onError like this the subscription is not cancelled on errors
        Fimber.e('Failed to observe cards', ex: ex, stacktrace: stack);
        return const DashboardLoadFailure();
      },
    );
  }
}
