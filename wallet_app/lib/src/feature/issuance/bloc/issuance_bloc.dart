import 'dart:async';

import 'package:collection/collection.dart';
import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/bloc/error_state.dart';
import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/model/flow_progress.dart';
import '../../../domain/model/issuance/continue_issuance_result.dart';
import '../../../domain/model/issuance/start_issuance_result.dart';
import '../../../domain/model/multiple_cards_flow.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/result/application_error.dart';
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
    final startResult = await startIssuanceUseCase.invoke(event.issuanceUri);
    _startIssuanceResult = startResult.value;
    await startResult.process(
      onSuccess: (result) {
        if (isRefreshFlow) {
          _emitStartOfRefreshFlow(emit, result);
        } else {
          emit(IssuanceCheckOrganization(organization: result.relyingParty));
        }
      },
      onError: (error) => emit(IssuanceGenericError(isRefreshFlow: isRefreshFlow, error: error)),
    );
  }

  void _emitStartOfRefreshFlow(emit, StartIssuanceResult result) {
    switch (result) {
      case StartIssuanceReadyToDisclose():
        emit(
          IssuanceProofIdentity(
            organization: result.relyingParty,
            policy: result.policy,
            requestedAttributes: result.requestedAttributes.values.flattened.toList(),
            isRefreshFlow: isRefreshFlow,
          ),
        );
      case StartIssuanceMissingAttributes():
        // We (currently) don't support this flow. (i.e. a refresh flow where-in the requested attributes changed
        // compared to the first issuance of this card, causing attributes to be missing this time around.)
        emit(
          IssuanceGenericError(
            error: GenericError(
              'Missing attributes detected during refresh flow',
              sourceError: Exception('Missing attributes: this is unexpected in the refresh flow'),
            ),
          ),
        );
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

  Future<void> _onIssuanceShareRequestedAttributesDeclined(event, emit) async => _stopIssuance(emit);

  Future<void> _onIssuanceShareRequestedAttributesApproved(event, emit) async {
    final state = this.state;
    if (state is! IssuanceProofIdentity) throw UnsupportedError('Incorrect state to $state');
    emit(IssuanceProvidePin(isRefreshFlow: isRefreshFlow));
  }

  Future<void> _onIssuancePinConfirmed(event, emit) async {
    final issuance = _startIssuanceResult;
    if (state is! IssuanceProvidePin) throw UnsupportedError('Incorrect state to $state');
    if (issuance == null) throw UnsupportedError('Can not move to select cards state without date');

    final continueResult = await continueIssuanceUseCase.invoke();
    _continueIssuanceResult = continueResult.value;

    await continueResult.process(
      onSuccess: (continueResult) {
        final issuedCards = continueResult.cards;
        if (issuedCards.length > 1) {
          emit(
            IssuanceSelectCards(
              isRefreshFlow: isRefreshFlow,
              multipleCardsFlow: MultipleCardsFlow.fromCards(
                issuedCards,
                issuance.relyingParty,
              ),
            ),
          );
        } else {
          emit(IssuanceCheckDataOffering(isRefreshFlow: isRefreshFlow, card: issuedCards.first));
        }
      },
      onError: (error) => emit(IssuanceGenericError(error: error)),
    );
  }

  Future<void> _onIssuanceCheckDataOfferingApproved(event, emit) async {
    final state = this.state;
    if (state is! IssuanceCheckDataOffering) throw UnsupportedError('Incorrect state: $state');
    final continueIssuanceResult = _continueIssuanceResult;
    if (continueIssuanceResult == null) throw UnsupportedError('ContinueIssuanceResult should be available');

    final acceptResult = await acceptIssuanceUseCase.invoke([continueIssuanceResult.cards.first.docType]);
    await acceptResult.process(
      onSuccess: (_) => emit(IssuanceCompleted(isRefreshFlow: isRefreshFlow, addedCards: continueIssuanceResult.cards)),
      onError: (error) => emit(IssuanceGenericError(error: error)),
    );
  }

  Future<void> _onIssuanceStopRequested(IssuanceStopRequested event, emit) async => _stopIssuance(emit);

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
        await _stopIssuance(emit);
      } else {
        //No more cards to check, add the selected ones and show completed state
        await _addCardsAndEmitCompleted(updatedMultipleCardFlow.selectedCards, emit);
      }
    }
  }

  Future<void> _addCardsAndEmitCompleted(List<WalletCard> selectedCards, emit) async {
    final acceptResult = await acceptIssuanceUseCase.invoke(selectedCards.map((e) => e.docType));
    await acceptResult.process(
      onSuccess: (_) => emit(IssuanceCompleted(isRefreshFlow: isRefreshFlow, addedCards: selectedCards)),
      onError: (error) => emit(IssuanceGenericError(isRefreshFlow: isRefreshFlow, error: error)),
    );
  }

  Future<void> _stopIssuance(emit) async {
    final cancelResult = await cancelIssuanceUseCase.invoke();
    await cancelResult.process(
      onSuccess: (_) => emit(IssuanceStopped(isRefreshFlow: isRefreshFlow)),
      onError: (error) => emit(IssuanceGenericError(error: error)),
    );
  }
}
