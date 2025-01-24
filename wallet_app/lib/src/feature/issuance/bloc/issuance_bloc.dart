import 'dart:async';

import 'package:collection/collection.dart';
import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/flow_progress.dart';
import '../../../domain/model/issuance/continue_issuance_result.dart';
import '../../../domain/model/issuance/start_issuance_result.dart';
import '../../../domain/model/multiple_cards_flow.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../domain/usecase/issuance/accept_issuance_usecase.dart';
import '../../../domain/usecase/issuance/cancel_issuance_usecase.dart';
import '../../../domain/usecase/issuance/continue_issuance_usecase.dart';
import '../../../domain/usecase/issuance/start_issuance_usecase.dart';
import '../../../util/extension/set_extension.dart';

part 'issuance_event.dart';
part 'issuance_state.dart';

class IssuanceBloc extends Bloc<IssuanceEvent, IssuanceState> {
  final StartIssuanceUseCase startIssuanceUseCase;
  final ContinueIssuanceUseCase continueIssuanceUseCase;
  final AcceptIssuanceUseCase acceptIssuanceUseCase;
  final CancelIssuanceUseCase cancelIssuanceUseCase;

  bool isRefreshFlow;
  StartIssuanceResult? _startIssuanceResult;
  ContinueIssuanceResult? _continueIssuanceResult;

  Organization? get organization => _startIssuanceResult?.relyingParty;

  IssuanceBloc(
    this.startIssuanceUseCase,
    this.continueIssuanceUseCase,
    this.acceptIssuanceUseCase,
    this.cancelIssuanceUseCase, {
    required this.isRefreshFlow,
  }) : super(const IssuanceInitial()) {
    on<IssuanceInitiated>(_onIssuanceInitiated);
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
    on<IssuanceUpdateState>((state, emit) => emit(state.state));
  }

  Future<void> _onIssuanceInitiated(IssuanceInitiated event, emit) async {
    try {
      final issuanceUri = event.issuanceUri;
      _startIssuanceResult = await startIssuanceUseCase.invoke(issuanceUri);
      if (isRefreshFlow) {
        /// We assume all the app is in [StartIssuanceReadyToDisclose] state for ALL refresh flows, as the
        /// requested attributes were available for the initial issuance. If that's somehow not the case the
        /// user will always be presented with the [IssuanceGenericError] state.
        final attributes = (_startIssuanceResult! as StartIssuanceReadyToDisclose).requestedAttributes;
        emit(
          IssuanceProofIdentity(
            organization: _startIssuanceResult!.relyingParty,
            policy: _startIssuanceResult!.policy,
            requestedAttributes: attributes.values.flattened.toList(),
            isRefreshFlow: isRefreshFlow,
          ),
        );
      } else {
        emit(IssuanceCheckOrganization(organization: _startIssuanceResult!.relyingParty));
      }
    } catch (ex) {
      Fimber.e('Failed to start issuance', ex: ex);
      emit(IssuanceGenericError(isRefreshFlow: isRefreshFlow));
    }
  }

  Future<void> _onIssuanceBackPressed(event, emit) async {
    final state = this.state;
    if (state.canGoBack) {
      if (state is IssuanceProofIdentity) {
        emit(
          IssuanceCheckOrganization(
            organization: _startIssuanceResult!.relyingParty,
            afterBackPressed: true,
          ),
        );
      }
      if (state is IssuanceProvidePin) {
        emit(
          IssuanceProofIdentity(
            isRefreshFlow: isRefreshFlow,
            afterBackPressed: true,
            organization: _startIssuanceResult!.relyingParty,
            policy: _startIssuanceResult!.policy,
            requestedAttributes:
                (_startIssuanceResult! as StartIssuanceReadyToDisclose).requestedAttributes.values.flattened.toList(),
          ),
        );
      }
      if (state is IssuanceCheckCards && state.multipleCardsFlow.isAtFirstCard) {
        emit(
          IssuanceSelectCards(
            isRefreshFlow: isRefreshFlow,
            multipleCardsFlow: state.multipleCardsFlow,
            showNoSelectionError: false,
            didGoBack: true,
          ),
        );
      }
      if (state is IssuanceCheckCards && !state.multipleCardsFlow.isAtFirstCard) {
        emit(
          IssuanceCheckCards(
            isRefreshFlow: isRefreshFlow,
            multipleCardsFlow: state.multipleCardsFlow.previous(),
            didGoBack: true,
          ),
        );
      }
    }
  }

  Future<void> _onIssuanceOrganizationApproved(event, emit) async {
    final state = this.state;
    if (state is! IssuanceCheckOrganization) throw UnsupportedError('Incorrect state to $state');
    final result = _startIssuanceResult;
    if (result == null) throw UnsupportedError('Bloc in incorrect state (no data loaded)');
    switch (result) {
      case StartIssuanceReadyToDisclose():
        emit(
          IssuanceProofIdentity(
            isRefreshFlow: false,
            organization: _startIssuanceResult!.relyingParty,
            policy: _startIssuanceResult!.policy,
            requestedAttributes: result.requestedAttributes.values.flattened.toList(),
          ),
        );
      case StartIssuanceMissingAttributes():
        emit(
          IssuanceMissingAttributes(
            isRefreshFlow: false,
            organization: _startIssuanceResult!.relyingParty,
            policy: _startIssuanceResult!.policy,
            missingAttributes: result.missingAttributes,
          ),
        );
    }
  }

  Future<void> _onIssuanceShareRequestedAttributesDeclined(event, emit) async {
    await cancelIssuanceUseCase.invoke();
    emit(IssuanceStopped(isRefreshFlow: isRefreshFlow));
  }

  Future<void> _onIssuanceShareRequestedAttributesApproved(event, emit) async {
    final state = this.state;
    if (state is! IssuanceProofIdentity) throw UnsupportedError('Incorrect state to $state');
    emit(IssuanceProvidePin(isRefreshFlow: isRefreshFlow));
  }

  Future<void> _onIssuancePinConfirmed(event, emit) async {
    final issuance = _startIssuanceResult;
    if (state is! IssuanceProvidePin) throw UnsupportedError('Incorrect state to $state');
    if (issuance == null) throw UnsupportedError('Can not move to select cards state without date');
    final result = _continueIssuanceResult = await continueIssuanceUseCase.invoke();
    if (result.cards.length > 1) {
      emit(
        IssuanceSelectCards(
          isRefreshFlow: isRefreshFlow,
          multipleCardsFlow: MultipleCardsFlow.fromCards(
            result.cards,
            issuance.relyingParty,
          ),
        ),
      );
    } else {
      emit(
        IssuanceCheckDataOffering(
          isRefreshFlow: isRefreshFlow,
          card: result.cards.first,
        ),
      );
    }
  }

  Future<void> _onIssuanceCheckDataOfferingApproved(event, emit) async {
    final state = this.state;
    if (state is! IssuanceCheckDataOffering) throw UnsupportedError('Incorrect state to $state');

    await acceptIssuanceUseCase.invoke([_continueIssuanceResult!.cards.first.docType]);
    emit(IssuanceCompleted(isRefreshFlow: isRefreshFlow, addedCards: _continueIssuanceResult!.cards));
  }

  Future<void> _onIssuanceStopRequested(IssuanceStopRequested event, emit) async {
    await cancelIssuanceUseCase.invoke();
    emit(IssuanceStopped(isRefreshFlow: isRefreshFlow));
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
      emit(
        IssuanceCheckCards(
          isRefreshFlow: isRefreshFlow,
          multipleCardsFlow: state.multipleCardsFlow,
        ),
      );
    }
  }

  FutureOr<void> _onIssuanceCardApproved(IssuanceCardApproved event, emit) async {
    final state = this.state;
    if (state is! IssuanceCheckCards) throw UnsupportedError('Incorrect state to $state');
    if (state.multipleCardsFlow.hasMoreCards) {
      emit(
        IssuanceCheckCards(
          isRefreshFlow: isRefreshFlow,
          multipleCardsFlow: state.multipleCardsFlow.next(),
        ),
      );
    } else {
      await _addCardsAndEmitCompleted(state.multipleCardsFlow.selectedCards, emit);
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
      emit(IssuanceCheckCards(isRefreshFlow: isRefreshFlow, multipleCardsFlow: updatedMultipleCardFlow));
    } else {
      if (updatedMultipleCardFlow.selectedCardIds.isEmpty) {
        //All cards are declined, show stopped.
        await cancelIssuanceUseCase.invoke();
        emit(IssuanceStopped(isRefreshFlow: isRefreshFlow));
      } else {
        //No more cards to check, add the selected ones and show completed state
        await _addCardsAndEmitCompleted(updatedMultipleCardFlow.selectedCards, emit);
      }
    }
  }

  Future<void> _addCardsAndEmitCompleted(List<WalletCard> selectedCards, emit) async {
    await acceptIssuanceUseCase.invoke(selectedCards.map((e) => e.docType));
    emit(IssuanceCompleted(isRefreshFlow: isRefreshFlow, addedCards: selectedCards));
  }
}
