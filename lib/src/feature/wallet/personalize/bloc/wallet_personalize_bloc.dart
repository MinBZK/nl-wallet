import 'dart:async';

import 'package:collection/collection.dart';
import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../domain/model/card_front.dart';
import '../../../../domain/model/issuance_response.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../../domain/usecase/card/get_pid_issuance_response_usecase.dart';
import '../../../../domain/usecase/card/get_wallet_cards_usecase.dart';
import '../../../../domain/usecase/card/wallet_add_issued_cards_usecase.dart';
import '../../../../domain/usecase/issuance/get_my_government_issuance_responses_usecase.dart';
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
    const mockDelay = kDebugMode ? kDefaultMockDelay : Duration(seconds: 8);
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
    if (state.canGoBack) {
      if (state is WalletPersonalizeScanId) {
        emit(const WalletPersonalizeScanIdIntro(afterBackPressed: true));
      }
    }
  }

  void _onRetrieveMoreCardsPressed(event, emit) async {
    emit(const WalletPersonalizeLoadInProgress(9.5));
    await Future.delayed(kDefaultMockDelay);

    final issuanceResponses = await getDemoWalletCardsIssuanceResponsesUseCase.invoke();
    final allCardIds = issuanceResponses.map((response) => response.cards).flattened.map((card) => card.id);
    emit(
      WalletPersonalizeSelectCards(
        issuanceResponses: issuanceResponses,
        selectedCardIds: allCardIds.toList(),
      ),
    );
  }

  void _onSelectedCardToggled(event, emit) async {
    final state = this.state;
    if (state is WalletPersonalizeSelectCards) {
      final currentSelection = Set<String>.from(state.selectedCardIds);
      if (currentSelection.contains(event.card.id)) {
        currentSelection.remove(event.card.id);
      } else {
        currentSelection.add(event.card.id);
      }
      emit(WalletPersonalizeSelectCards(
        issuanceResponses: state.issuanceResponses,
        selectedCardIds: currentSelection.toList(),
      ));
    }
  }

  void _onAddSelectedCardsPressed(event, emit) async {
    final state = this.state;
    if (state is WalletPersonalizeSelectCards) {
      emit(const WalletPersonalizeLoadInProgress(8));
      await Future.delayed(kDefaultMockDelay);
      try {
        for (final response in state.issuanceResponses) {
          final cardsToAdd = response.cards.where((card) => state.selectedCardIds.contains(card.id));
          final organization = response.organization;
          await walletAddIssuedCardsUseCase.invoke(cardsToAdd.toList(), organization);
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
      final cards = await getWalletCardsUseCase.getWalletCardsOrderedByIdAsc();
      emit(WalletPersonalizeSuccess(cards));
    } catch (ex, stack) {
      Fimber.e('Failed to fetch cards from wallet', ex: ex, stacktrace: stack);
      emit(WalletPersonalizeFailure());
    }
  }
}
