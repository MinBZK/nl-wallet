import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/wallet_card_summary.dart';
import '../../../../domain/usecase/card/get_wallet_card_summary_usecase.dart';

part 'card_summary_event.dart';
part 'card_summary_state.dart';

class CardSummaryBloc extends Bloc<CardSummaryEvent, CardSummaryState> {
  final GetWalletCardSummaryUseCase getWalletCardSummaryUseCase;

  CardSummaryBloc(this.getWalletCardSummaryUseCase) : super(CardSummaryInitial()) {
    on<CardSummaryLoadTriggered>(_onCardSummaryLoadTriggered);
  }

  void _onCardSummaryLoadTriggered(CardSummaryLoadTriggered event, emit) async {
    emit(const CardSummaryLoadInProgress());
    try {
      WalletCardSummary summary = await getWalletCardSummaryUseCase.getWalletCardSummary(event.cardId);
      emit(CardSummaryLoadSuccess(summary));
    } catch (error) {
      emit(const CardSummaryLoadFailure());
    }
  }
}
