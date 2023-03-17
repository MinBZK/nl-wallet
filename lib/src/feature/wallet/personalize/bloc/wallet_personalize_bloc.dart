import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../domain/model/card_front.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../../domain/usecase/card/get_pid_issuance_response_usecase.dart';
import '../../../../domain/usecase/card/get_wallet_cards_usecase.dart';
import '../../../../domain/usecase/card/wallet_add_issued_cards_usecase.dart';
import '../../../../domain/usecase/issuance/get_my_government_issuance_responses_usecase.dart';
import '../../../../wallet_constants.dart';

part 'wallet_personalize_event.dart';
part 'wallet_personalize_state.dart';

class WalletPersonalizeBloc extends Bloc<WalletPersonalizeEvent, WalletPersonalizeState> {
  final GetPidIssuanceResponseUseCase getPidIssuanceResponseUseCase;
  final GetMyGovernmentIssuanceResponsesUseCase getDemoWalletCardsIssuanceResponsesUseCase;
  final WalletAddIssuedCardsUseCase walletAddIssuedCardsUseCase;
  final GetWalletCardsUseCase getWalletCardsUseCase;

  WalletPersonalizeBloc(
    this.getPidIssuanceResponseUseCase,
    this.walletAddIssuedCardsUseCase,
    this.getDemoWalletCardsIssuanceResponsesUseCase,
    this.getWalletCardsUseCase,
  ) : super(WalletPersonalizeInitial()) {
    on<WalletPersonalizeLoginWithDigidClicked>(_onLoginWithDigidClicked);
    on<WalletPersonalizeLoginWithDigidSucceeded>(_onLoginWithDigidSucceeded);
    on<WalletPersonalizeLoginWithDigidFailed>(_onLoginWithDigidFailed);
    on<WalletPersonalizeOfferingVerified>(_onOfferingVerified);
    on<WalletPersonalizePinConfirmed>(_onPinConfirmed);
    on<WalletPersonalizeOnBackPressed>(_onBackPressed);
    on<WalletPersonalizeOnRetryClicked>(_onRetryClicked);
  }

  void _onLoginWithDigidClicked(event, emit) async => emit(WalletPersonalizeLoadingPid());

  void _onLoginWithDigidSucceeded(event, emit) async {
    try {
      final issuanceResponse = await getPidIssuanceResponseUseCase.invoke();
      final card = issuanceResponse.cards.first;
      emit(WalletPersonalizeCheckData(availableAttributes: card.attributes.toList()));
    } catch (ex, stack) {
      Fimber.e('Failed to get PID', ex: ex, stacktrace: stack);
      emit(WalletPersonalizeFailure());
    }
  }

  void _onLoginWithDigidFailed(event, emit) async => emit(WalletPersonalizeDigidFailure());

  void _onOfferingVerified(WalletPersonalizeOfferingVerified event, emit) async {
    emit(const WalletPersonalizeConfirmPin());
  }

  void _onRetryClicked(event, emit) async => emit(WalletPersonalizeInitial());

  void _onBackPressed(event, emit) async {
    final state = this.state;
    if (state.canGoBack) {
      if (state is WalletPersonalizeConfirmPin) {
        final issuanceResponse = await getPidIssuanceResponseUseCase.invoke();
        final card = issuanceResponse.cards.first;
        emit(
          WalletPersonalizeCheckData(
            didGoBack: true,
            availableAttributes: card.attributes.toList(),
          ),
        );
      }
    }
  }

  Future<void> _onPinConfirmed(event, emit) async {
    final state = this.state;
    if (state is WalletPersonalizeConfirmPin) {
      emit(const WalletPersonalizeLoadInProgress(5));
      await Future.delayed(kDefaultMockDelay);
      try {
        final issuanceResponse = await getPidIssuanceResponseUseCase.invoke();
        await walletAddIssuedCardsUseCase.invoke(issuanceResponse.cards, issuanceResponse.organization);
        await _loadCardsAndEmitSuccessState(event, emit);
      } catch (ex, stack) {
        Fimber.e('Failed to add cards to wallet', ex: ex, stacktrace: stack);
        emit(WalletPersonalizeFailure());
      }
    }
  }

  Future<void> _loadCardsAndEmitSuccessState(event, emit) async {
    try {
      final cards = await getWalletCardsUseCase.invoke();
      emit(WalletPersonalizeSuccess(cards));
    } catch (ex, stack) {
      Fimber.e('Failed to fetch cards from wallet', ex: ex, stacktrace: stack);
      emit(WalletPersonalizeFailure());
    }
  }
}
