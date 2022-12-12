import 'package:collection/collection.dart';
import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/timeline_attribute.dart';
import '../../../domain/usecase/card/log_card_interaction_usecase.dart';
import '../../../domain/usecase/verification/get_verification_request_usecase.dart';
import '../../../domain/usecase/wallet/get_requested_attributes_from_wallet_usecase.dart';
import '../../../wallet_constants.dart';
import '../model/organization.dart';
import '../model/verification_flow.dart';

part 'verification_event.dart';
part 'verification_state.dart';

class VerificationBloc extends Bloc<VerificationEvent, VerificationState> {
  final GetVerificationRequestUseCase getVerificationRequestUseCase;
  final LogCardInteractionUseCase logCardInteractionUseCase;
  final GetRequestedAttributesFromWalletUseCase getRequestedAttributesFromWalletUseCase;

  VerificationBloc(
      this.getVerificationRequestUseCase, this.getRequestedAttributesFromWalletUseCase, this.logCardInteractionUseCase)
      : super(VerificationInitial()) {
    on<VerificationLoadRequested>(_onVerificationLoadRequested);
    on<VerificationOrganizationApproved>(_onVerificationOrganizationApproved);
    on<VerificationShareRequestedAttributesApproved>(_onVerificationShareRequestedAttributesApproved);
    on<VerificationPinConfirmed>(_onVerificationPinConfirmed);
    on<VerificationBackPressed>(_onVerificationBackPressed);
    on<VerificationStopRequested>(_onVerificationStopRequested);
  }

  void _onVerificationLoadRequested(VerificationLoadRequested event, emit) async {
    if (state is VerificationInitial) {
      try {
        emit(VerificationLoadInProgress());
        final request = await getVerificationRequestUseCase.invoke(event.sessionId);
        emit(
          VerificationCheckOrganization(
            VerificationFlow(
              id: request.id,
              organization: request.organization,
              attributes: await getRequestedAttributesFromWalletUseCase.invoke(request.requestedAttributes),
              policy: request.policy,
            ),
          ),
        );
      } catch (ex, stack) {
        Fimber.e('Failed to load VerificationRequest for id ${event.sessionId}', ex: ex, stacktrace: stack);
        emit(VerificationGenericError());
      }
    }
  }

  void _onVerificationOrganizationApproved(VerificationOrganizationApproved event, emit) {
    final state = this.state;
    if (state is VerificationCheckOrganization) {
      if (state.flow.hasMissingAttributes) {
        emit(VerificationMissingAttributes(state.flow));
      } else {
        emit(VerificationConfirmDataAttributes(state.flow));
      }
    }
  }

  void _onVerificationShareRequestedAttributesApproved(VerificationShareRequestedAttributesApproved event, emit) {
    final state = this.state;
    if (state is VerificationConfirmDataAttributes) emit(VerificationConfirmPin(state.flow));
  }

  void _onVerificationPinConfirmed(VerificationPinConfirmed event, emit) async {
    final state = this.state;
    if (state is VerificationConfirmPin) {
      emit(VerificationLoadInProgress());
      if (event.flow != null) _logCardInteraction(event.flow!, InteractionType.success);
      await Future.delayed(kDefaultMockDelay);
      emit(VerificationSuccess(state.flow));
    }
  }

  void _onVerificationBackPressed(VerificationBackPressed event, emit) {
    final state = this.state;
    if (state.canGoBack) {
      if (state is VerificationConfirmDataAttributes) {
        emit(VerificationCheckOrganization(state.flow, afterBackPressed: true));
      } else if (state is VerificationMissingAttributes) {
        emit(VerificationCheckOrganization(state.flow, afterBackPressed: true));
      } else if (state is VerificationConfirmPin) {
        emit(VerificationConfirmDataAttributes(state.flow, afterBackPressed: true));
      }
    }
  }

  void _onVerificationStopRequested(VerificationStopRequested event, emit) async {
    emit(VerificationLoadInProgress());
    if (event.flow != null) _logCardInteraction(event.flow!, InteractionType.rejected);
    await Future.delayed(kDefaultMockDelay);
    emit(const VerificationStopped());
  }

  void _logCardInteraction(VerificationFlow flow, InteractionType type) {
    final attributesByCardId = flow.resolvedAttributes.groupListsBy((element) => element.sourceCardId);
    attributesByCardId.forEach((cardId, attributes) {
      logCardInteractionUseCase.invoke(type, cardId, flow.organization, attributes);
    });
  }
}
