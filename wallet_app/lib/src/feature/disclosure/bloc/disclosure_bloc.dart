import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../domain/usecase/disclosure/cancel_disclosure_usecase.dart';
import '../../../domain/usecase/disclosure/start_disclosure_usecase.dart';
import '../../../util/extension/bloc_extension.dart';
import '../../report_issue/report_issue_screen.dart';

part 'disclosure_event.dart';
part 'disclosure_state.dart';

class DisclosureBloc extends Bloc<DisclosureEvent, DisclosureState> {
  final StartDisclosureUseCase _startDisclosureUseCase;
  final CancelDisclosureUseCase _cancelDisclosureUseCase;

  StartDisclosureResult? _startDisclosureResult;
  StreamSubscription? _startDisclosureStreamSubscription;

  Organization? get relyingParty => _startDisclosureResult?.relyingParty;

  DisclosureBloc(
    this._startDisclosureUseCase,
    this._cancelDisclosureUseCase,
  ) : super(DisclosureLoadInProgress()) {
    on<DisclosureSessionStarted>(_onSessionStarted);
    on<DisclosureStopRequested>(_onStopRequested);
    on<DisclosureBackPressed>(_onBackPressed);
    on<DisclosureOrganizationApproved>(_onOrganizationApproved);
    on<DisclosureShareRequestedAttributesApproved>(_onShareRequestedAttributesApproved);
    on<DisclosurePinConfirmed>(_onPinConfirmed);
    on<DisclosureReportPressed>(_onReportPressed);
  }

  void _onSessionStarted(DisclosureSessionStarted event, Emitter<DisclosureState> emit) async {
    try {
      _startDisclosureResult = await _startDisclosureUseCase.invoke(event.uri);
      emit(
        DisclosureCheckOrganization(
          relyingParty: _startDisclosureResult!.relyingParty,
          originUrl: _startDisclosureResult!.originUrl,
          isFirstInteractionWithOrganization: _startDisclosureResult!.isFirstInteractionWithOrganization,
        ),
      );
    } catch (ex) {
      Fimber.e('Failed to start disclosure', ex: ex);
      await handleError(
        ex,
        onNetworkError: (error, hasInternet) => emit(DisclosureNetworkError(hasInternet: hasInternet)),
        onUnhandledError: (error) => emit(DisclosureGenericError()),
      );
    }
  }

  void _onStopRequested(DisclosureStopRequested event, emit) async {
    try {
      emit(DisclosureLoadInProgress());
      await _cancelDisclosureUseCase.invoke();
    } catch (ex) {
      Fimber.e('Failed to explicitly cancel disclosure flow', ex: ex);
    } finally {
      emit(const DisclosureStopped());
    }
  }

  void _onBackPressed(DisclosureBackPressed event, emit) async {
    final state = this.state;
    if (state is DisclosureConfirmDataAttributes) {
      assert(_startDisclosureResult != null, 'StartDisclosureResult should always be available at this stage');
      emit(
        DisclosureCheckOrganization(
          relyingParty: state.relyingParty,
          originUrl: _startDisclosureResult!.originUrl,
          isFirstInteractionWithOrganization: _startDisclosureResult?.isFirstInteractionWithOrganization == true,
          afterBackPressed: true,
        ),
      );
    } else if (state is DisclosureMissingAttributes) {
      assert(_startDisclosureResult != null, 'StartDisclosureResult should always be available at this stage');
      emit(
        DisclosureCheckOrganization(
          relyingParty: state.relyingParty,
          originUrl: _startDisclosureResult!.originUrl,
          isFirstInteractionWithOrganization: _startDisclosureResult?.isFirstInteractionWithOrganization == true,
          afterBackPressed: true,
        ),
      );
    } else if (state is DisclosureConfirmPin) {
      assert(_startDisclosureResult is StartDisclosureReadyToDisclose, 'Invalid state');
      final result = _startDisclosureResult as StartDisclosureReadyToDisclose;
      emit(
        DisclosureConfirmDataAttributes(
          relyingParty: _startDisclosureResult!.relyingParty,
          requestedAttributes: result.requestedAttributes,
          policy: result.policy,
          requestPurpose: result.requestPurpose,
          afterBackPressed: true,
        ),
      );
    }
  }

  void _onOrganizationApproved(DisclosureOrganizationApproved event, emit) async {
    final startDisclosureResult = _startDisclosureResult;
    switch (startDisclosureResult) {
      case null:
        throw UnsupportedError('Organization approved while in invalid state, i.e. no result available!');
      case StartDisclosureReadyToDisclose():
        emit(
          DisclosureConfirmDataAttributes(
            relyingParty: startDisclosureResult.relyingParty,
            requestPurpose: startDisclosureResult.requestPurpose,
            requestedAttributes: startDisclosureResult.requestedAttributes,
            policy: startDisclosureResult.policy,
          ),
        );
      case StartDisclosureMissingAttributes():
        emit(
          DisclosureMissingAttributes(
            relyingParty: startDisclosureResult.relyingParty,
            missingAttributes: startDisclosureResult.missingAttributes,
          ),
        );
    }
  }

  void _onShareRequestedAttributesApproved(DisclosureShareRequestedAttributesApproved event, emit) {
    assert(_startDisclosureResult is StartDisclosureReadyToDisclose, 'Invalid data state to continue disclosing');
    assert(state is DisclosureConfirmDataAttributes, 'Invalid UI state to move to pin entry');
    if (state is DisclosureConfirmDataAttributes) emit(const DisclosureConfirmPin());
  }

  void _onPinConfirmed(DisclosurePinConfirmed event, emit) {
    assert(_startDisclosureResult != null, 'DisclosureResult should still be available after confirming the tx');
    emit(DisclosureSuccess(relyingParty: _startDisclosureResult!.relyingParty, returnUrl: event.returnUrl));
  }

  void _onReportPressed(DisclosureReportPressed event, Emitter<DisclosureState> emit) async {
    Fimber.d('User selected reporting option ${event.option}');
    try {
      emit(DisclosureLoadInProgress());
      await _cancelDisclosureUseCase.invoke();
    } catch (ex) {
      Fimber.e('Failed to explicitly cancel disclosure flow', ex: ex);
    } finally {
      emit(const DisclosureLeftFeedback());
    }
  }

  @override
  Future<void> close() async {
    _startDisclosureStreamSubscription?.cancel();
    if (state is DisclosureConfirmPin) {
      /// The bloc being closed while in the [DisclosureConfirmPin] indicates the user has provided
      /// an incorrect pin too many times, causing the [DisclosureScreen] and thus this bloc to be
      /// closed. To avoid any issues with future disclosure we make sure to cancel the onGoing flow
      /// here.
      try {
        await _cancelDisclosureUseCase.invoke();
      } catch (ex) {
        Fimber.e('Failed to explicitly cancel disclosure', ex: ex);
      }
    }
    return super.close();
  }
}
