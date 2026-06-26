import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/card/wallet_card.dart';
import '../../../../domain/model/event/wallet_event.dart';
import '../../../../domain/usecase/card/get_wallet_card_usecase.dart';
import '../../../../domain/usecase/event/get_wallet_events_for_card_usecase.dart';
import '../../../../domain/usecase/event/get_wallet_events_pid_usecase.dart';
import '../../../../domain/usecase/pid/check_is_pid.dart';

part 'card_history_event.dart';
part 'card_history_state.dart';

class CardHistoryBloc extends Bloc<CardHistoryEvent, CardHistoryState> {
  final GetWalletCardUseCase _getWalletCardUseCase;
  final GetWalletEventsForCardUseCase _getEventsForCardUseCase;
  final GetWalletEventsForPidUseCase _getEventsForPidUseCase;
  final CheckIsPidUseCase _checkIsPidUseCase;

  CardHistoryBloc(
    this._getWalletCardUseCase,
    this._getEventsForCardUseCase,
    this._getEventsForPidUseCase,
    this._checkIsPidUseCase,
  ) : super(CardHistoryInitial()) {
    on<CardHistoryLoadTriggered>(_onCardHistoryLoadTriggered);
  }

  Future<void> _onCardHistoryLoadTriggered(CardHistoryLoadTriggered event, emit) async {
    emit(const CardHistoryLoadInProgress());
    try {
      final cardResult = await _getWalletCardUseCase.invoke(event.attestationId);
      final isPid = await _checkIsPidUseCase.invoke(cardResult.value!);

      final events = isPid.value!
          ? await _getEventsForPidUseCase.invoke()
          : await _getEventsForCardUseCase.invoke(event.attestationId);

      emit(CardHistoryLoadSuccess(cardResult.value!, events.value!));
    } catch (e) {
      emit(const CardHistoryLoadFailure());
    }
  }
}
