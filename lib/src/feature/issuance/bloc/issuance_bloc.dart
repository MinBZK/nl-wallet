import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/issuance_flow.dart';
import '../../../domain/model/timeline_attribute.dart';
import '../../../domain/usecase/card/log_card_interaction_usecase.dart';
import '../../../domain/usecase/card/wallet_add_issued_card_usecase.dart';
import '../../../domain/usecase/issuance/get_issuance_response_usecase.dart';
import '../../../domain/usecase/wallet/get_requested_attributes_from_wallet_usecase.dart';
import '../../../wallet_constants.dart';
import '../../verification/model/organization.dart';

part 'issuance_event.dart';
part 'issuance_state.dart';

class IssuanceBloc extends Bloc<IssuanceEvent, IssuanceState> {
  final GetIssuanceResponseUseCase getIssuanceResponseUseCase;
  final GetRequestedAttributesFromWalletUseCase getRequestedAttributesFromWalletUseCase;
  final WalletAddIssuedCardUseCase walletAddIssuedCardUseCase;
  final LogCardInteractionUseCase logCardInteractionUseCase;

  IssuanceBloc(
    this.getIssuanceResponseUseCase,
    this.walletAddIssuedCardUseCase,
    this.getRequestedAttributesFromWalletUseCase,
    this.logCardInteractionUseCase,
  ) : super(IssuanceInitial()) {
    on<IssuanceLoadTriggered>(_onIssuanceLoadTriggered);
    on<IssuanceBackPressed>(_onIssuanceBackPressed);
    on<IssuanceOrganizationDeclined>(_onIssuanceOrganizationDeclined);
    on<IssuanceOrganizationApproved>(_onIssuanceOrganizationApproved);
    on<IssuanceShareRequestedAttributesDeclined>(_onIssuanceShareRequestedAttributesDeclined);
    on<IssuanceShareRequestedAttributesApproved>(_onIssuanceShareRequestedAttributesApproved);
    on<IssuancePinConfirmed>(_onIssuancePinConfirmed);
    on<IssuanceCheckDataOfferingApproved>(_onIssuanceCheckDataOfferingApproved);
    on<IssuanceStopRequested>(_onIssuanceStopRequested);
  }

  void _onIssuanceBackPressed(event, emit) async {
    final state = this.state;
    if (state.canGoBack) {
      if (state is IssuanceProofIdentity) emit(IssuanceCheckOrganization(state.flow, afterBackPressed: true));
      if (state is IssuanceProvidePin) emit(IssuanceProofIdentity(state.flow, afterBackPressed: true));
    }
  }

  void _onIssuanceLoadTriggered(event, emit) async {
    emit(IssuanceLoadInProgress());
    await Future.delayed(kDefaultMockDelay);
    final response = await getIssuanceResponseUseCase.invoke(event.sessionId);
    emit(
      IssuanceCheckOrganization(
        IssuanceFlow(
          organization: response.organization,
          attributes: await getRequestedAttributesFromWalletUseCase.invoke(response.requestedAttributes),
          cards: response.cards,
        ),
      ),
    );
  }

  void _onIssuanceOrganizationDeclined(event, emit) async {
    emit(IssuanceLoadInProgress());
    await Future.delayed(kDefaultMockDelay);
    emit(IssuanceStopped());
  }

  void _onIssuanceOrganizationApproved(event, emit) async {
    final state = this.state;
    if (state is! IssuanceCheckOrganization) throw UnsupportedError('Incorrect state to $state');
    emit(IssuanceProofIdentity(state.flow));
  }

  void _onIssuanceShareRequestedAttributesDeclined(event, emit) async {
    emit(IssuanceLoadInProgress());
    await Future.delayed(kDefaultMockDelay);
    emit(IssuanceStopped());
  }

  void _onIssuanceShareRequestedAttributesApproved(event, emit) async {
    final state = this.state;
    if (state is! IssuanceProofIdentity) throw UnsupportedError('Incorrect state to $state');
    emit(IssuanceProvidePin(state.flow));
  }

  void _onIssuancePinConfirmed(event, emit) async {
    final state = this.state;
    if (state is! IssuanceProvidePin) throw UnsupportedError('Incorrect state to $state');
    emit(IssuanceCheckDataOffering(state.flow));
  }

  void _onIssuanceCheckDataOfferingApproved(event, emit) async {
    final state = this.state;
    if (state is! IssuanceCheckDataOffering) throw UnsupportedError('Incorrect state to $state');
    _logCardInteraction(state.flow, InteractionType.success);
    await walletAddIssuedCardUseCase.invoke(state.flow.cards.first);
    emit(IssuanceCardAdded(state.flow));
  }

  void _onIssuanceStopRequested(IssuanceStopRequested event, emit) async {
    if (event.flow != null) {
      bool userAlreadySharedData = state is IssuanceCheckDataOffering;
      _logCardInteraction(event.flow!, userAlreadySharedData ? InteractionType.success : InteractionType.rejected);
    }
    emit(IssuanceLoadInProgress());
    await Future.delayed(kDefaultMockDelay);
    emit(IssuanceStopped());
  }

  void _logCardInteraction(IssuanceFlow flow, InteractionType type) {
    final usedCardIds = flow.resolvedAttributes.map((e) => e.sourceCardId).toSet();
    for (var cardId in usedCardIds) {
      logCardInteractionUseCase.invoke(cardId, type, flow.organization.shortName);
    }
  }
}
