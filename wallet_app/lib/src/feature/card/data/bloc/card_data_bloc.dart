import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/card/wallet_card.dart';
import '../../../../domain/usecase/card/observe_wallet_card_usecase.dart';

part 'card_data_event.dart';
part 'card_data_state.dart';

class CardDataBloc extends Bloc<CardDataEvent, CardDataState> {
  final ObserveWalletCardUseCase observeWalletCardUseCase;

  CardDataBloc(this.observeWalletCardUseCase) : super(CardDataInitial()) {
    on<CardDataLoadTriggered>(_onCardDataLoadTriggered);
  }

  Future<void> _onCardDataLoadTriggered(CardDataLoadTriggered event, emit) async {
    if (state is! CardDataLoadSuccess) emit(const CardDataLoadInProgress());
    await emit.forEach(
      observeWalletCardUseCase.invoke(event.cardId),
      // ignore: unnecessary_lambdas, not actually unnecessary due to expected signature
      onData: (data) => CardDataLoadSuccess(data),
      onError: (ex, stack) {
        //Note: when providing onError like this the subscription is not cancelled on errors
        Fimber.e('Failed to observe card', ex: ex, stacktrace: stack);
        return const CardDataLoadFailure();
      },
    );
  }
}
