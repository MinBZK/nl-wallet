import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/wallet_card_detail.dart';
import '../../../../domain/usecase/card/get_wallet_card_detail_usecase.dart';

part 'card_detail_event.dart';
part 'card_detail_state.dart';

class CardDetailBloc extends Bloc<CardDetailEvent, CardDetailState> {
  final GetWalletCardDetailUseCase getWalletCardDetailUseCase;

  CardDetailBloc(this.getWalletCardDetailUseCase) : super(CardDetailInitial()) {
    on<CardDetailLoadTriggered>(_onCardSummaryLoadTriggered);
  }

  void _onCardSummaryLoadTriggered(CardDetailLoadTriggered event, emit) async {
    emit(const CardDetailLoadInProgress());
    try {
      WalletCardDetail detail = await getWalletCardDetailUseCase.invoke(event.cardId);
      emit(CardDetailLoadSuccess(detail));
    } catch (error) {
      emit(CardDetailLoadFailure(event.cardId));
    }
  }
}
