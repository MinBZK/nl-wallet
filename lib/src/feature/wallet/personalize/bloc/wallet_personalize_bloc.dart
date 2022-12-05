import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/card_front.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../../domain/usecase/card/get_pid_card_usecase.dart';
import '../../../../domain/usecase/card/wallet_add_issued_card_usecase.dart';

part 'wallet_personalize_event.dart';
part 'wallet_personalize_state.dart';

class WalletPersonalizeBloc extends Bloc<WalletPersonalizeEvent, WalletPersonalizeState> {
  final GetPidCardUseCase getPidCardUseCase;
  final WalletAddIssuedCardUseCase walletAddIssuedCardUseCase;

  WalletPersonalizeBloc(this.getPidCardUseCase, this.walletAddIssuedCardUseCase) : super(WalletPersonalizeInitial()) {
    on<WalletPersonalizeLoginWithDigidClicked>(_onLoginWithDigidClicked);
    on<WalletPersonalizeLoginWithDigidSucceeded>(_onLoginWithDigidSucceeded);
    on<WalletPersonalizeOfferingAccepted>(_onOfferingAccepted);
    on<WalletPersonalizeOnRetryClicked>(_onRetryClicked);
  }

  Future<void> _onLoginWithDigidClicked(event, emit) async {
    emit(WalletPersonalizeLoadingPid());
  }

  Future<void> _onLoginWithDigidSucceeded(event, emit) async {
    try {
      final card = await getPidCardUseCase.invoke();
      emit(WalletPersonalizeCheckData(card));
    } catch (ex, stack) {
      Fimber.e('Failed to get PID', ex: ex, stacktrace: stack);
      emit(WalletPersonalizeFailure());
    }
  }

  Future<void> _onOfferingAccepted(WalletPersonalizeOfferingAccepted event, emit) async {
    try {
      await walletAddIssuedCardUseCase.invoke(event.acceptedCard);
      emit(WalletPersonalizeSuccess(event.acceptedCard));
    } catch (ex, stack) {
      Fimber.e('Failed create PID card', ex: ex, stacktrace: stack);
      emit(WalletPersonalizeFailure());
    }
  }

  Future<void> _onRetryClicked(event, emit) async {
    emit(WalletPersonalizeInitial());
  }
}
