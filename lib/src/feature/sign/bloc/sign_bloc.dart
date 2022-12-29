import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/timeline/timeline_attribute.dart';
import '../../../domain/usecase/card/log_card_signing_usecase.dart';
import '../../../domain/usecase/sign/get_sign_request_usecase.dart';
import '../../../domain/usecase/wallet/get_requested_attributes_from_wallet_usecase.dart';
import '../../../wallet_constants.dart';
import '../model/sign_flow.dart';

part 'sign_event.dart';
part 'sign_state.dart';

class SignBloc extends Bloc<SignEvent, SignState> {
  final GetSignRequestUseCase getSignRequestUseCase;
  final LogCardSigningUseCase logCardSigningUseCase;
  final GetRequestedAttributesFromWalletUseCase getRequestedAttributesFromWalletUseCase;

  SignBloc(
    this.getRequestedAttributesFromWalletUseCase,
    this.logCardSigningUseCase,
    this.getSignRequestUseCase,
  ) : super(const SignInitial()) {
    on<SignLoadTriggered>(_onLoadTriggered);
    on<SignOrganizationApproved>(_onOrganizationApproved);
    on<SignAgreementChecked>(_onAgreementChecked);
    on<SignAgreementApproved>(_onAgreementApproved);
    on<SignPinConfirmed>(_onPinConfirmed);
    on<SignStopRequested>(_onStopRequested);
    on<SignBackPressed>(_onBackPressed);
  }

  void _onLoadTriggered(SignLoadTriggered event, emit) async {
    emit(SignLoadInProgress(state.flow));
    try {
      final request = await getSignRequestUseCase.invoke(event.id);
      emit(
        SignCheckOrganization(
          SignFlow(
            id: request.id,
            organization: request.organization,
            trustProvider: request.trustProvider,
            document: request.document,
            attributes: await getRequestedAttributesFromWalletUseCase.invoke(request.requestedAttributes),
            policy: request.policy,
          ),
        ),
      );
    } catch (ex, stack) {
      Fimber.e('Failed to load sign flow', ex: ex, stacktrace: stack);
      emit(SignError(state.flow));
    }
  }

  void _onOrganizationApproved(event, emit) async {
    assert(state is SignCheckOrganization);
    try {
      emit(SignCheckAgreement(state.flow!));
    } catch (ex) {
      Fimber.e('Failed to move to SignCheckAgreement state', ex: ex);
      emit(SignError(state.flow));
    }
  }

  void _onAgreementChecked(event, emit) async {
    assert(state is SignCheckAgreement);
    try {
      emit(SignConfirmAgreement(state.flow!));
    } catch (ex) {
      Fimber.e('Failed to move to SignConfirmAgreement state', ex: ex);
      emit(SignError(state.flow));
    }
  }

  void _onAgreementApproved(event, emit) async {
    assert(state is SignConfirmAgreement);
    try {
      emit(SignConfirmPin(state.flow!));
    } catch (ex) {
      Fimber.e('Failed to move to SignConfirmPin state', ex: ex);
      emit(SignError(state.flow));
    }
  }

  void _onPinConfirmed(event, emit) async {
    assert(state is SignConfirmPin);
    emit(SignLoadInProgress(state.flow));
    await Future.delayed(kDefaultMockDelay);
    try {
      _logCardInteraction(state.flow!, SigningStatus.success);
      emit(SignSuccess(state.flow!));
    } catch (ex) {
      Fimber.e('Failed to move to SignSuccess state', ex: ex);
      emit(SignError(state.flow));
    }
  }

  void _onStopRequested(event, emit) async {
    assert(state.flow != null, 'Stop can only be requested after flow is loaded');
    emit(SignLoadInProgress(state.flow));
    await Future.delayed(kDefaultMockDelay);
    try {
      _logCardInteraction(state.flow!, SigningStatus.rejected);
      emit(SignStopped(state.flow!));
    } catch (ex) {
      Fimber.e('Failed to move to SignStopped state', ex: ex);
      emit(SignError(state.flow));
    }
  }

  void _onBackPressed(event, emit) async {
    final state = this.state;
    if (state.canGoBack) {
      if (state is SignCheckAgreement) {
        emit(SignCheckOrganization(state.flow, afterBackPressed: true));
      } else if (state is SignConfirmAgreement) {
        emit(SignCheckAgreement(state.flow, afterBackPressed: true));
      } else if (state is SignConfirmPin) {
        emit(SignConfirmAgreement(state.flow, afterBackPressed: true));
      }
    }
  }

  void _logCardInteraction(SignFlow flow, SigningStatus status) {
    logCardSigningUseCase.invoke(
      status,
      flow.policy,
      flow.organization,
      flow.document,
      flow.resolvedAttributes,
    );
  }
}
