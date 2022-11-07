import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/data_attribute.dart';
import '../../../../domain/usecase/card/get_wallet_card_data_attributes_usecase.dart';

part 'card_data_event.dart';
part 'card_data_state.dart';

class CardDataBloc extends Bloc<CardDataEvent, CardDataState> {
  final GetWalletCardDataAttributesUseCase getWalletCardDataAttributesUseCase;

  CardDataBloc(this.getWalletCardDataAttributesUseCase) : super(CardDataInitial()) {
    on<CardDataLoadTriggered>(_onCardDataLoadTriggered);
  }

  void _onCardDataLoadTriggered(CardDataLoadTriggered event, emit) async {
    emit(const CardDataLoadInProgress());
    try {
      List<DataAttribute> results = await getWalletCardDataAttributesUseCase.invoke(event.cardId);
      emit(CardDataLoadSuccess(results));
    } catch (error) {
      emit(const CardDataLoadFailure());
    }
  }
}
