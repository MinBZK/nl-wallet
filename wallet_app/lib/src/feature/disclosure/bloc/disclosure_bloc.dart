import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/bloc/error_state.dart';
import '../../../domain/model/bloc/network_error_state.dart';
import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/model/disclosure/disclosure_session_type.dart';
import '../../../domain/model/event/wallet_event.dart';
import '../../../domain/model/flow_progress.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/result/application_error.dart';
import '../../../domain/usecase/disclosure/cancel_disclosure_usecase.dart';
import '../../../domain/usecase/disclosure/start_disclosure_usecase.dart';
import '../../../domain/usecase/event/get_most_recent_wallet_event_usecase.dart';
import '../../../util/cast_util.dart';
import '../../report_issue/report_issue_screen.dart';

part 'disclosure_event.dart';
part 'disclosure_state.dart';

class DisclosureBloc extends Bloc<DisclosureEvent, DisclosureState> {
  final StartDisclosureUseCase _startDisclosureUseCase;
  final CancelDisclosureUseCase _cancelDisclosureUseCase;
  final GetMostRecentWalletEventUseCase _getMostRecentWalletEventUseCase;

  StartDisclosureResult? _startDisclosureResult;

  Organization? get relyingParty => _startDisclosureResult?.relyingParty;

  bool get isLoginFlow => tryCast<StartDisclosureReadyToDisclose>(_startDisclosureResult)?.type == DisclosureType.login;

  DisclosureBloc(
    this._startDisclosureUseCase,
    this._cancelDisclosureUseCase,
    this._getMostRecentWalletEventUseCase,
  ) : super(DisclosureInitial()) {
    on<DisclosureSessionStarted>(_onSessionStarted);
    on<DisclosureStopRequested>(_onStopRequested);
    on<DisclosureBackPressed>(_onBackPressed);
    on<DisclosureOrganizationApproved>(_onOrganizationApproved);
    on<DisclosureShareRequestedAttributesApproved>(_onShareRequestedAttributesApproved);
    on<DisclosurePinConfirmed>(_onPinConfirmed);
    on<DisclosureReportPressed>(_onReportPressed);
    on<DisclosureConfirmPinFailed>(_onConfirmPinFailed);
  }

  Future<void> _onSessionStarted(DisclosureSessionStarted event, Emitter<DisclosureState> emit) async {
    // Cancel any potential ongoing disclosure, this can happen when a second disclosure
    // deeplink is pressed while the disclosure flow is currently open. This opens a second
    // disclosure bloc before the original one is closed, thus we need to cancel it here.
    await _cancelDisclosureUseCase.invoke();
    final startDisclosureResult = await _startDisclosureUseCase.invoke(event.uri, isQrCode: event.isQrCode);

    await startDisclosureResult.process(
      onSuccess: (result) {
        _startDisclosureResult = result; // Cache the result
        if (isLoginFlow) {
          _emitDisclosureResultForLogin(result, emit);
        } else {
          emit(
            DisclosureCheckOrganization(
              relyingParty: result.relyingParty,
              originUrl: result.originUrl,
              sharedDataWithOrganizationBefore: result.sharedDataWithOrganizationBefore,
              sessionType: result.sessionType,
            ),
          );
        }
      },
      onError: (error) {
        Fimber.e('Failed to start disclosure', ex: error);
        switch (error) {
          case GenericError():
            emit(DisclosureGenericError(error: error, returnUrl: error.redirectUrl));
          case NetworkError():
            emit(DisclosureNetworkError(hasInternet: error.hasInternet, error: error));
          case SessionError():
            _handleSessionError(emit, error);
          case ExternalScannerError():
            emit(DisclosureExternalScannerError(error: error));
          default:
            emit(DisclosureGenericError(error: error));
        }
      },
    );
  }

  void _emitDisclosureResultForLogin(StartDisclosureResult result, Emitter<DisclosureState> emit) {
    switch (result) {
      case StartDisclosureReadyToDisclose():
        emit(
          DisclosureCheckOrganizationForLogin(
            relyingParty: result.relyingParty,
            originUrl: result.originUrl,
            sessionType: result.sessionType,
            policy: result.policy,
            sharedDataWithOrganizationBefore: result.sharedDataWithOrganizationBefore,
            requestedAttributes: result.requestedAttributes,
          ),
        );
      case StartDisclosureMissingAttributes():
        emit(
          DisclosureGenericError(
            error: GenericError(
              'Disclosing unavailable attributes in login flow is not supported',
              sourceError: Exception('Missing attributes: this unexpected in login flow'),
            ),
          ),
        );
    }
  }

  Future<void> _onStopRequested(DisclosureStopRequested event, emit) async {
    emit(DisclosureLoadInProgress(state.stepperProgress));
    final cancelResult = await _cancelDisclosureUseCase.invoke();

    // Check edge case where relyingParty (needed to render stopped screen) is not available.
    if (_startDisclosureResult?.relyingParty == null) {
      emit(
        DisclosureGenericError(
          error:
              GenericError('Invalid state, no relyingParty to render', sourceError: Exception('Relying party is null')),
          returnUrl: cancelResult.value,
        ),
      );
      return;
    }

    await cancelResult.process(
      onSuccess: (returnUrl) {
        emit(
          DisclosureStopped(
            organization: _startDisclosureResult!.relyingParty,
            isLoginFlow: isLoginFlow,
            returnUrl: returnUrl,
          ),
        );
      },
      onError: (error) {
        switch (error) {
          case NetworkError():
            emit(DisclosureNetworkError(error: error, hasInternet: error.hasInternet));
          default:
            emit(DisclosureGenericError(error: error, returnUrl: cancelResult.value));
        }
      },
    );
  }

  Future<void> _onBackPressed(DisclosureBackPressed event, emit) async {
    final state = this.state;
    if (state is DisclosureConfirmDataAttributes) {
      assert(_startDisclosureResult != null, 'StartDisclosureResult should always be available at this stage');
      // No need to check for login flow as [DisclosureConfirmDataAttributes] is never used there
      emit(
        DisclosureCheckOrganization(
          relyingParty: state.relyingParty,
          sharedDataWithOrganizationBefore: _startDisclosureResult?.sharedDataWithOrganizationBefore ?? false,
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
          sharedDataWithOrganizationBefore: _startDisclosureResult?.sharedDataWithOrganizationBefore ?? false,
          sessionType: _startDisclosureResult!.sessionType,
          afterBackPressed: true,
        ),
      );
    } else if (state is DisclosureConfirmPin) {
      assert(_startDisclosureResult is StartDisclosureReadyToDisclose, 'Invalid state');
      final result = _startDisclosureResult! as StartDisclosureReadyToDisclose;
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

  Future<void> _onOrganizationApproved(DisclosureOrganizationApproved event, emit) async {
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

  Future<void> _onPinConfirmed(DisclosurePinConfirmed event, emit) async {
    assert(_startDisclosureResult != null, 'DisclosureResult should still be available after confirming the tx');
    final lastEvent = await _getMostRecentWalletEventUseCase.invoke();
    assert(lastEvent != null, 'Last event should not be null after a successful disclosure');
    emit(
      DisclosureSuccess(
        relyingParty: _startDisclosureResult!.relyingParty,
        event: lastEvent,
        returnUrl: event.returnUrl,
        isLoginFlow: isLoginFlow,
      ),
    );
  }

  Future<void> _onReportPressed(DisclosureReportPressed event, Emitter<DisclosureState> emit) async {
    Fimber.d('User selected reporting option ${event.option}');
    emit(DisclosureLoadInProgress(state.stepperProgress));
    final cancelResult = await _cancelDisclosureUseCase.invoke();
    if (cancelResult.hasError) Fimber.e('Failed to explicitly cancel disclosure flow', ex: cancelResult.error);
    emit(DisclosureLeftFeedback(returnUrl: cancelResult.value));
  }

  Future<void> _onConfirmPinFailed(DisclosureConfirmPinFailed event, Emitter<DisclosureState> emit) async {
    emit(DisclosureLoadInProgress(state.stepperProgress));
    // {event.error} is the error that was thrown when user tried to confirm the disclosure session with a PIN.
    switch (event.error) {
      case SessionError():
        _handleSessionError(emit, event.error as SessionError);
        return;
      case NetworkError():
        await _cancelDisclosureUseCase.invoke(); // Attempt to cancel the session, but propagate original error
        emit(DisclosureNetworkError(error: event.error, hasInternet: (event.error as NetworkError).hasInternet));
        return;
      default:
        Fimber.d('Disclosure failed unexpectedly when entering pin, cause: ${event.error}.');
    }
    // Call cancelSession to avoid stale session and to try and provide more context (e.g. returnUrl).
    final cancelResult = await _cancelDisclosureUseCase.invoke();
    await cancelResult.process(
      onSuccess: (returnUrl) => emit(DisclosureGenericError(error: event.error, returnUrl: returnUrl)),
      onError: (error) => emit(DisclosureGenericError(error: error)),
    );
  }

  void _handleSessionError(Emitter<DisclosureState> emit, SessionError error) {
    final isCrossDevice = _startDisclosureResult?.sessionType == DisclosureSessionType.crossDevice;
    switch (error.state) {
      case SessionState.expired:
        emit(
          DisclosureSessionExpired(
            error: error,
            canRetry: error.canRetry,
            isCrossDevice: isCrossDevice,
            returnUrl: error.returnUrl,
          ),
        );
      case SessionState.cancelled:
        emit(
          DisclosureCancelledSessionError(
            error: error,
            relyingParty: relyingParty,
            returnUrl: error.returnUrl,
          ),
        );
    }
  }
}
