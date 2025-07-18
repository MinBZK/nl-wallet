import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:rxdart/rxdart.dart';

import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/model/event/wallet_event.dart';
import '../../../domain/model/update/version_state.dart';
import '../../../domain/usecase/card/observe_wallet_cards_usecase.dart';
import '../../../domain/usecase/event/observe_recent_wallet_events_usecase.dart';

part 'dashboard_event.dart';
part 'dashboard_state.dart';

class DashboardBloc extends Bloc<DashboardEvent, DashboardState> {
  final ObserveWalletCardsUseCase _observeWalletCardsUseCase;
  final ObserveRecentWalletEventsUseCase _observeRecentWalletEventsUseCase;

  DashboardBloc(
    this._observeWalletCardsUseCase,
    this._observeRecentWalletEventsUseCase,
    List<WalletCard>? preloadedCards,
  ) : super(preloadedCards == null ? const DashboardStateInitial() : DashboardLoadSuccess(cards: preloadedCards)) {
    on<DashboardLoadTriggered>(_onCardOverviewLoadTriggered, transformer: restartable());
  }

  Future<void> _onCardOverviewLoadTriggered(DashboardLoadTriggered event, emit) async {
    if (state is! DashboardLoadSuccess || event.forceRefresh) emit(const DashboardLoadInProgress());
    await emit.forEach(
      CombineLatestStream.combine2(
        _observeWalletCardsUseCase.invoke(),
        _observeRecentWalletEventsUseCase.invoke(),
        (cards, history) => (cards, history),
      ),
      onData: (data) {
        final (cards, history) = data;
        return DashboardLoadSuccess(
          cards: cards,
          history: history,
        );
      },
      onError: (ex, stack) {
        //Note: when providing onError like this the subscription is not cancelled on errors
        Fimber.e('Failed to observe cards', ex: ex, stacktrace: stack);
        return const DashboardLoadFailure();
      },
    );
  }
}
