import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../domain/model/card_front.dart';
import '../../../../domain/model/multiple_cards_flow.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../../domain/usecase/card/get_pid_issuance_response_usecase.dart';
import '../../../../domain/usecase/card/get_wallet_cards_usecase.dart';
import '../../../../domain/usecase/card/wallet_add_issued_cards_usecase.dart';
import '../../../../domain/usecase/issuance/get_my_government_issuance_responses_usecase.dart';
import '../../../../util/extension/set_extensions.dart';
import '../../../../wallet_constants.dart';
import '../../../verification/model/organization.dart';

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
    on<WalletPersonalizeOfferingVerified>(_onOfferingVerified);
    on<WalletPersonalizeScanInitiated>(_onScanInitiated);
    on<WalletPersonalizeScanEvent>(_onScanEvent);
    on<WalletPersonalizePhotoApproved>(_onPhotoApproved);
    on<WalletPersonalizeOnRetryClicked>(_onRetryClicked);
    on<WalletPersonalizeOnBackPressed>(_onBackPressed);
    on<WalletPersonalizeRetrieveMoreCardsPressed>(_onRetrieveMoreCardsPressed);
    on<WalletPersonalizeSelectedCardToggled>(_onSelectedCardToggled);
    on<WalletPersonalizeAddSelectedCardsPressed>(_onAddSelectedCardsPressed);
    on<WalletPersonalizeDataOnCardDeclined>(_onDataOnCardDeclined);
    on<WalletPersonalizeDataOnCardConfirmed>(_onDataOnCardConfirmed);
    on<WalletPersonalizePinConfirmed>(_onPinConfirmed);
    on<WalletPersonalizeSkipRetrieveMoreCardsPressed>(_loadCardsAndEmitSuccessState);
    on<WalletPersonalizeSkipAddMoreCardsPressed>(_loadCardsAndEmitSuccessState);
  }

  void _onLoginWithDigidClicked(event, emit) async {
    emit(WalletPersonalizeLoadingPid());
  }

  void _onLoginWithDigidSucceeded(event, emit) async {
    emit(const WalletPersonalizeScanIdIntro());
  }

  void _onOfferingVerified(WalletPersonalizeOfferingVerified event, emit) async {
    try {
      final issuanceResponse = await getPidIssuanceResponseUseCase.invoke();
      final organization = issuanceResponse.organization;
      await walletAddIssuedCardsUseCase.invoke(issuanceResponse.cards, organization);
      emit(WalletPersonalizeRetrieveMoreCards());
    } catch (ex, stack) {
      Fimber.e('Failed create PID card', ex: ex, stacktrace: stack);
      emit(WalletPersonalizeFailure());
    }
  }

  void _onScanInitiated(event, emit) async {
    emit(WalletPersonalizeScanId());
  }

  void _onScanEvent(event, emit) async {
    const mockDelay = kDebugMode ? kDefaultMockDelay : Duration(seconds: 3);
    emit(const WalletPersonalizeLoadingPhoto(mockDelay));
    await Future.delayed(mockDelay);

    try {
      final issuanceResponse = await getPidIssuanceResponseUseCase.invoke();
      final card = issuanceResponse.cards.first;
      emit(WalletPersonalizeCheckData(availableAttributes: card.attributes.toList()));
    } catch (ex, stack) {
      Fimber.e('Failed to get PID', ex: ex, stacktrace: stack);
      emit(WalletPersonalizeFailure());
    }
  }

  void _onPhotoApproved(event, emit) async {
    try {
      final issuanceResponse = await getPidIssuanceResponseUseCase.invoke();
      final organization = issuanceResponse.organization;
      await walletAddIssuedCardsUseCase.invoke(issuanceResponse.cards, organization);
      emit(WalletPersonalizeRetrieveMoreCards());
    } catch (ex, stack) {
      Fimber.e('Failed create PID card', ex: ex, stacktrace: stack);
      emit(WalletPersonalizeFailure());
    }
  }

  void _onRetryClicked(event, emit) async {
    emit(WalletPersonalizeInitial());
  }

  void _onBackPressed(event, emit) async {
    final state = this.state;
    if (state.canGoBack) {
      if (state is WalletPersonalizeScanId) {
        emit(const WalletPersonalizeScanIdIntro(afterBackPressed: true));
      }
      if (state is WalletPersonalizeCheckCards) {
        if (!state.multipleCardsFlow.isAtFirstCard) {
          emit(state.copyForPreviousCard());
        } else {
          emit(WalletPersonalizeSelectCards(
            didGoBack: true,
            multipleCardsFlow: state.multipleCardsFlow,
          ));
        }
      }
      if (state is WalletPersonalizeConfirmPin) {
        emit(WalletPersonalizeCheckCards(
          didGoBack: true,
          multipleCardsFlow: state.multipleCardsFlow,
        ));
      }
    }
  }

  void _onRetrieveMoreCardsPressed(event, emit) async {
    emit(const WalletPersonalizeLoadInProgress(9.5));
    await Future.delayed(kDefaultMockDelay);

    final issuanceResponses = await getDemoWalletCardsIssuanceResponsesUseCase.invoke();
    emit(
      WalletPersonalizeSelectCards(multipleCardsFlow: MultipleCardsFlow.fromIssuance(issuanceResponses)),
    );
  }

  void _onSelectedCardToggled(event, emit) async {
    final state = this.state;
    if (state is WalletPersonalizeSelectCards) {
      emit(state.toggleCard(event.card.id));
    }
  }

  void _onAddSelectedCardsPressed(event, emit) async {
    final state = this.state;
    if (state is WalletPersonalizeSelectCards) {
      if (state.selectedCards.isEmpty) {
        emit(WalletPersonalizeSelectCards(multipleCardsFlow: state.multipleCardsFlow, showNoSelectionError: true));
      } else {
        emit(
          WalletPersonalizeCheckCards(multipleCardsFlow: state.multipleCardsFlow),
        );
      }
    }
  }

  Future<void> _onDataOnCardDeclined(event, emit) async {
    final state = this.state;
    if (state is! WalletPersonalizeCheckCards) throw UnsupportedError('Unsupported state to decline cards: $state');
    final selectedCardIds = Set<String>.from(state.multipleCardsFlow.selectedCardIds);
    selectedCardIds.remove(state.cardToCheck.id);
    final updatedMultipleCardFlow = state.multipleCardsFlow.copyWith(selectedCardIds: selectedCardIds);
    if (state.hasMoreCards) {
      //activeIndex is maintained, but since the selected set is now shorter the next card is now the activeCard.
      emit(WalletPersonalizeCheckCards(multipleCardsFlow: updatedMultipleCardFlow));
    } else {
      if (updatedMultipleCardFlow.selectedCardIds.isEmpty) {
        //All cards are declined, skip to end.
        _loadCardsAndEmitSuccessState(event, emit);
      } else {
        //No more cards to check, let user enter pin to confirm adding the approved ones.
        emit(WalletPersonalizeConfirmPin(multipleCardsFlow: updatedMultipleCardFlow));
      }
    }
  }

  Future<void> _onDataOnCardConfirmed(event, emit) async {
    final state = this.state;
    if (state is WalletPersonalizeCheckCards) {
      if (state.hasMoreCards) {
        emit(state.copyForNextCard());
      } else {
        emit(WalletPersonalizeConfirmPin(multipleCardsFlow: state.multipleCardsFlow));
      }
    }
  }

  Future<void> _onPinConfirmed(event, emit) async {
    final state = this.state;
    if (state is WalletPersonalizeConfirmPin) {
      emit(const WalletPersonalizeLoadInProgress(12.5));
      await Future.delayed(kDefaultMockDelay);
      try {
        for (final card in state.multipleCardsFlow.selectedCards) {
          final organization = state.multipleCardsFlow.cardToOrganizations[card];
          await walletAddIssuedCardsUseCase.invoke([card], organization!);
        }
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
