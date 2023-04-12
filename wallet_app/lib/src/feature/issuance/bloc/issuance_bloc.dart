import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/issuance_flow.dart';
import '../../../domain/model/multiple_cards_flow.dart';
import '../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../domain/usecase/card/log_card_interaction_usecase.dart';
import '../../../domain/usecase/card/wallet_add_issued_cards_usecase.dart';
import '../../../domain/usecase/issuance/get_issuance_response_usecase.dart';
import '../../../domain/usecase/wallet/get_requested_attributes_from_wallet_usecase.dart';
import '../../../util/extension/set_extensions.dart';
import '../../../wallet_constants.dart';
import '../../verification/model/organization.dart';

part 'issuance_event.dart';
part 'issuance_state.dart';

class IssuanceBloc extends Bloc<IssuanceEvent, IssuanceState> {
  final GetIssuanceResponseUseCase getIssuanceResponseUseCase;
  final GetRequestedAttributesFromWalletUseCase getRequestedAttributesFromWalletUseCase;
  final WalletAddIssuedCardsUseCase walletAddIssuedCardsUseCase;
  final LogCardInteractionUseCase logCardInteractionUseCase;

  bool _userSharedData = false;

  IssuanceBloc(
    this.getIssuanceResponseUseCase,
    this.walletAddIssuedCardsUseCase,
    this.getRequestedAttributesFromWalletUseCase,
    this.logCardInteractionUseCase,
  ) : super(const IssuanceInitial(false)) {
    on<IssuanceLoadTriggered>(_onIssuanceLoadTriggered);
    on<IssuanceBackPressed>(_onIssuanceBackPressed);
    on<IssuanceOrganizationApproved>(_onIssuanceOrganizationApproved);
    on<IssuanceShareRequestedAttributesDeclined>(_onIssuanceShareRequestedAttributesDeclined);
    on<IssuanceShareRequestedAttributesApproved>(_onIssuanceShareRequestedAttributesApproved);
    on<IssuancePinConfirmed>(_onIssuancePinConfirmed);
    on<IssuanceCheckDataOfferingApproved>(_onIssuanceCheckDataOfferingApproved);
    on<IssuanceCardToggled>(_onIssuanceCardToggled);
    on<IssuanceSelectedCardsConfirmed>(_onIssuanceSelectedCardsConfirmed);
    on<IssuanceCardDeclined>(_onIssuanceCardDeclined);
    on<IssuanceCardApproved>(_onIssuanceCardApproved);
    on<IssuanceStopRequested>(_onIssuanceStopRequested);
  }

  void _onIssuanceBackPressed(event, emit) async {
    final state = this.state;
    if (state.canGoBack) {
      if (state is IssuanceProofIdentity) {
        emit(IssuanceCheckOrganization(state.isRefreshFlow, state.flow, afterBackPressed: true));
      }
      if (state is IssuanceProvidePin) {
        emit(IssuanceProofIdentity(state.isRefreshFlow, state.flow, afterBackPressed: true));
      }
      if (state is IssuanceCheckCards && state.multipleCardsFlow.isAtFirstCard) {
        emit(IssuanceSelectCards(
          state.isRefreshFlow,
          state.flow,
          state.multipleCardsFlow,
          didGoBack: true,
        ));
      }
      if (state is IssuanceCheckCards && !state.multipleCardsFlow.isAtFirstCard) {
        emit(IssuanceCheckCards(
          state.isRefreshFlow,
          flow: state.flow,
          multipleCardsFlow: state.multipleCardsFlow.previous(),
          didGoBack: true,
        ));
      }
    }
  }

  void _onIssuanceLoadTriggered(IssuanceLoadTriggered event, emit) async {
    emit(IssuanceLoadInProgress(event.isRefreshFlow));

    await Future.delayed(kDefaultMockDelay);
    final response = await getIssuanceResponseUseCase.invoke(event.sessionId);
    final attributes = await getRequestedAttributesFromWalletUseCase.invoke(response.requestedAttributes);
    final IssuanceFlow flow = IssuanceFlow(
      organization: response.organization,
      attributes: attributes,
      policy: response.policy,
      cards: response.cards,
    );

    if (event.isRefreshFlow) {
      emit(IssuanceProofIdentity(event.isRefreshFlow, flow));
    } else {
      emit(IssuanceCheckOrganization(event.isRefreshFlow, flow));
    }
  }

  void _onIssuanceOrganizationApproved(event, emit) async {
    final state = this.state;
    if (state is! IssuanceCheckOrganization) throw UnsupportedError('Incorrect state to $state');
    emit(IssuanceProofIdentity(state.isRefreshFlow, state.flow));
  }

  void _onIssuanceShareRequestedAttributesDeclined(event, emit) async {
    emit(IssuanceStopped(state.isRefreshFlow));
  }

  void _onIssuanceShareRequestedAttributesApproved(event, emit) async {
    final state = this.state;
    if (state is! IssuanceProofIdentity) throw UnsupportedError('Incorrect state to $state');
    emit(IssuanceProvidePin(state.isRefreshFlow, state.flow));
  }

  void _onIssuancePinConfirmed(event, emit) async {
    final state = this.state;
    if (state is! IssuanceProvidePin) throw UnsupportedError('Incorrect state to $state');
    _userSharedData = true;
    if (state.flow.cards.length > 1) {
      emit(
        IssuanceSelectCards(
          state.isRefreshFlow,
          state.flow,
          MultipleCardsFlow.fromCards(state.flow.cards, state.flow.organization),
        ),
      );
    } else {
      emit(IssuanceCheckDataOffering(state.isRefreshFlow, state.flow));
    }
  }

  void _onIssuanceCheckDataOfferingApproved(event, emit) async {
    final state = this.state;
    if (state is! IssuanceCheckDataOffering) throw UnsupportedError('Incorrect state to $state');
    _logCardInteraction(state.flow, InteractionStatus.success);
    await walletAddIssuedCardsUseCase.invoke(state.flow.cards, state.flow.organization);
    emit(IssuanceCompleted(state.isRefreshFlow, state.flow, state.flow.cards));
  }

  void _onIssuanceStopRequested(IssuanceStopRequested event, emit) async {
    if (event.flow != null) {
      _logCardInteraction(event.flow!, _userSharedData ? InteractionStatus.success : InteractionStatus.rejected);
    }
    emit(IssuanceStopped(state.isRefreshFlow));
  }

  void _logCardInteraction(IssuanceFlow flow, InteractionStatus status) {
    logCardInteractionUseCase.invoke(status, flow.policy, flow.organization, flow.resolvedAttributes);
  }

  FutureOr<void> _onIssuanceCardToggled(IssuanceCardToggled event, emit) {
    final state = this.state;
    if (state is! IssuanceSelectCards) throw UnsupportedError('Incorrect state to $state');
    emit(state.toggleCard(event.card.id));
  }

  FutureOr<void> _onIssuanceSelectedCardsConfirmed(IssuanceSelectedCardsConfirmed event, emit) {
    final state = this.state;
    if (state is! IssuanceSelectCards) throw UnsupportedError('Incorrect state to $state');
    if (state.selectedCards.isEmpty) {
      emit(state.copyWith(showNoSelectionError: true));
    } else {
      emit(IssuanceCheckCards(
        state.isRefreshFlow,
        flow: state.flow,
        multipleCardsFlow: state.multipleCardsFlow,
      ));
    }
  }

  FutureOr<void> _onIssuanceCardApproved(IssuanceCardApproved event, emit) async {
    final state = this.state;
    if (state is! IssuanceCheckCards) throw UnsupportedError('Incorrect state to $state');
    if (state.multipleCardsFlow.hasMoreCards) {
      emit(IssuanceCheckCards(
        state.isRefreshFlow,
        flow: state.flow,
        multipleCardsFlow: state.multipleCardsFlow.next(),
      ));
    } else {
      await _addCardsAndEmitCompleted(state.flow, state.multipleCardsFlow.selectedCards, emit);
    }
  }

  FutureOr<void> _onIssuanceCardDeclined(IssuanceCardDeclined event, emit) async {
    final state = this.state;
    if (state is! IssuanceCheckCards) throw UnsupportedError('Incorrect state to $state');
    final selectedCardIds = Set<String>.from(state.multipleCardsFlow.selectedCardIds);
    selectedCardIds.remove(event.card.id);
    final updatedMultipleCardFlow = state.multipleCardsFlow.copyWith(selectedCardIds: selectedCardIds);
    if (state.multipleCardsFlow.hasMoreCards) {
      //activeIndex is maintained, but since the selected set is now shorter the next card is now the activeCard.
      emit(IssuanceCheckCards(state.isRefreshFlow, flow: state.flow, multipleCardsFlow: updatedMultipleCardFlow));
    } else {
      if (updatedMultipleCardFlow.selectedCardIds.isEmpty) {
        //All cards are declined, show stopped.
        emit(IssuanceStopped(state.isRefreshFlow));
      } else {
        //No more cards to check, add the selected ones and show completed state
        await _addCardsAndEmitCompleted(state.flow, updatedMultipleCardFlow.selectedCards, emit);
      }
    }
  }

  Future<void> _addCardsAndEmitCompleted(IssuanceFlow flow, List<WalletCard> selectedCards, emit) async {
    _logCardInteraction(flow, InteractionStatus.success);
    await walletAddIssuedCardsUseCase.invoke(selectedCards, flow.organization);
    emit(IssuanceCompleted(state.isRefreshFlow, flow, selectedCards));
  }
}
