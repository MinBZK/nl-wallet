import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/bloc/error_state.dart';
import '../../../domain/model/bloc/network_error_state.dart';
import '../../../domain/model/disclosure/disclosure_session_type.dart';
import '../../../domain/model/disclosure/disclosure_type.dart';
import '../../../domain/model/flow_progress.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../domain/usecase/disclosure/cancel_disclosure_usecase.dart';
import '../../../domain/usecase/disclosure/start_disclosure_usecase.dart';
import '../../../util/cast_util.dart';
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

  bool get isLoginFlow => tryCast<StartDisclosureReadyToDisclose>(_startDisclosureResult)?.type == DisclosureType.login;

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
    on<DisclosureConfirmPinFailed>(_onConfirmPinFailed);
  }

  void _onSessionStarted(DisclosureSessionStarted event, Emitter<DisclosureState> emit) async {
    try {
      // Cancel any potential ongoing disclosure, this can happen when a second disclosure
      // deeplink is pressed while the disclosure flow is currently open. This opens a second
      // disclosure bloc before the original one is closed, thus we need to cancel it here.
      await _cancelDisclosureUseCase.invoke();
      final startDisclosureResult =
          _startDisclosureResult = await _startDisclosureUseCase.invoke(event.uri, event.isQrCode);
      if (startDisclosureResult is StartDisclosureReadyToDisclose && isLoginFlow) {
        emit(
          DisclosureCheckOrganizationForLogin(
            relyingParty: startDisclosureResult.relyingParty,
            originUrl: startDisclosureResult.originUrl,
            sessionType: startDisclosureResult.sessionType,
            policy: startDisclosureResult.policy,
            sharedDataWithOrganizationBefore: startDisclosureResult.sharedDataWithOrganizationBefore,
            requestedAttributes: startDisclosureResult.requestedAttributes,
          ),
        );
      } else {
        emit(
          DisclosureCheckOrganization(
            relyingParty: startDisclosureResult.relyingParty,
            originUrl: startDisclosureResult.originUrl,
            sharedDataWithOrganizationBefore: startDisclosureResult.sharedDataWithOrganizationBefore,
            sessionType: startDisclosureResult.sessionType,
          ),
        );
      }
    } catch (ex) {
      Fimber.e('Failed to start disclosure', ex: ex);
      await handleError(
        ex,
        onNetworkError: (error, hasInternet) => emit(DisclosureNetworkError(hasInternet: hasInternet, error: error)),
        onUnhandledError: (error) => emit(DisclosureGenericError(error: error)),
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
      final relyingParty = _startDisclosureResult?.relyingParty;
      if (relyingParty == null) {
        emit(const DisclosureGenericError(error: 'Invalid state, no relying party to render stopped'));
      } else {
        emit(
          DisclosureStopped(
              organization: _startDisclosureResult!.relyingParty,
              isLoginFlow: isLoginFlow,
              returnUrl: null // See PVW-2577,
              ),
        );
      }
    }
  }

  void _onBackPressed(DisclosureBackPressed event, emit) async {
    final state = this.state;
    if (state is DisclosureConfirmDataAttributes) {
      assert(_startDisclosureResult != null, 'StartDisclosureResult should always be available at this stage');
      // No need to check for login flow as [DisclosureConfirmDataAttributes] is never used there
      emit(
        DisclosureCheckOrganization(
          relyingParty: state.relyingParty,
          sharedDataWithOrganizationBefore: _startDisclosureResult?.sharedDataWithOrganizationBefore == true,
          sessionType: _startDisclosureResult!.sessionType,
          originUrl: _startDisclosureResult!.originUrl,
          afterBackPressed: true,
        ),
      );
    } else if (state is DisclosureMissingAttributes) {
      assert(_startDisclosureResult != null, 'StartDisclosureResult should always be available at this stage');
      // No need to check for login flow as [DisclosureMissingAttributes] is never used there (bsn always available)
      emit(
        DisclosureCheckOrganization(
          relyingParty: state.relyingParty,
          originUrl: _startDisclosureResult!.originUrl,
          sharedDataWithOrganizationBefore: _startDisclosureResult?.sharedDataWithOrganizationBefore == true,
          sessionType: _startDisclosureResult!.sessionType,
          afterBackPressed: true,
        ),
      );
    } else if (state is DisclosureConfirmPin) {
      assert(_startDisclosureResult is StartDisclosureReadyToDisclose, 'Invalid state');
      final result = _startDisclosureResult as StartDisclosureReadyToDisclose;
      if (state.isLoginFlow) {
        emit(
          DisclosureCheckOrganizationForLogin(
            relyingParty: result.relyingParty,
            originUrl: result.originUrl,
            sessionType: result.sessionType,
            policy: result.policy,
            sharedDataWithOrganizationBefore: result.sharedDataWithOrganizationBefore,
            requestedAttributes: result.requestedAttributes,
            afterBackPressed: true,
          ),
        );
      } else {
        emit(
          DisclosureConfirmDataAttributes(
            relyingParty: result.relyingParty,
            requestedAttributes: result.requestedAttributes,
            policy: result.policy,
            requestPurpose: result.requestPurpose,
            afterBackPressed: true,
          ),
        );
      }
    }
  }

  void _onOrganizationApproved(DisclosureOrganizationApproved event, emit) async {
    final startDisclosureResult = _startDisclosureResult;
    switch (startDisclosureResult) {
      case null:
        throw UnsupportedError('Organization approved while in invalid state, i.e. no result available!');
      case StartDisclosureReadyToDisclose():
        if (startDisclosureResult.type == DisclosureType.login) {
          // When the user is in the login flow, skip straight to the enter pin screen
          emit(
            DisclosureConfirmPin(
              relyingParty: startDisclosureResult.relyingParty,
              isLoginFlow: true,
            ),
          );
        } else {
          // When the user is sharing other attributes, ask the user to confirm them
          emit(
            DisclosureConfirmDataAttributes(
              relyingParty: startDisclosureResult.relyingParty,
              requestPurpose: startDisclosureResult.requestPurpose,
              requestedAttributes: startDisclosureResult.requestedAttributes,
              policy: startDisclosureResult.policy,
            ),
          );
        }
      case StartDisclosureMissingAttributes():
        // When the user doesn't have all the requested attributes, present the ones that are missing
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
    if (state is DisclosureConfirmDataAttributes) {
      final relyingParty = (state as DisclosureConfirmDataAttributes).relyingParty;
      emit(DisclosureConfirmPin(relyingParty: relyingParty));
    }
  }

  void _onPinConfirmed(DisclosurePinConfirmed event, emit) {
    assert(_startDisclosureResult != null, 'DisclosureResult should still be available after confirming the tx');
    emit(
      DisclosureSuccess(
        relyingParty: _startDisclosureResult!.relyingParty,
        returnUrl: event.returnUrl,
        isLoginFlow: isLoginFlow,
      ),
    );
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

  void _onConfirmPinFailed(DisclosureConfirmPinFailed event, Emitter<DisclosureState> emit) async {
    try {
      emit(DisclosureLoadInProgress());
      await _cancelDisclosureUseCase.invoke();
    } catch (ex) {
      Fimber.e('Failed to explicitly cancel disclosure flow', ex: ex);
    } finally {
      await handleError(
        event.error,
        onNetworkError: (ex, hasInternet) => emit(DisclosureNetworkError(error: ex, hasInternet: hasInternet)),
        onUnhandledError: (ex) => emit(DisclosureGenericError(error: ex)),
      );
    }
  }

  @override
  Future<void> close() async {
    _startDisclosureStreamSubscription?.cancel();
    return super.close();
  }
}
