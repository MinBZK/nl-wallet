import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/bloc/error_state.dart';
import '../../../domain/model/bloc/network_error_state.dart';
import '../../../domain/model/disclosure/disclose_card_request.dart';
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
import '../../../util/extension/list_extension.dart';
import '../../report_issue/report_issue_screen.dart';

part 'disclosure_event.dart';
part 'disclosure_state.dart';

class DisclosureBloc extends Bloc<DisclosureEvent, DisclosureState> {
  /// Use case responsible for initiating a disclosure session.
  final StartDisclosureUseCase _startDisclosureUseCase;

  /// Use case responsible for canceling an ongoing disclosure session.
  final CancelDisclosureUseCase _cancelDisclosureUseCase;

  /// Use case to retrieve the most recent wallet event after disclosure completion.
  final GetMostRecentWalletEventUseCase _getMostRecentWalletEventUseCase;

  /// Stores the result of a successful [StartDisclosureUseCase] invocation.
  /// Used to track session state and relay data between bloc methods.
  StartDisclosureResult? _startDisclosureResult;

  /// A cached version of the user's card selection, used when navigating back and forth from the selection page.
  List<DiscloseCardRequest>? _cardRequestsSelectionCache;

  /// Returns the relying party organization from the cached [StartDisclosureResult].
  Organization? get relyingParty => _startDisclosureResult?.relyingParty;

  /// Determines if the current flow is a login type disclosure.
  bool get isLoginFlow {
    assert(_startDisclosureResult != null, '_startDisclosureResult should be set to correctly fetch isLoginFlow');
    return tryCast<StartDisclosureReadyToDisclose>(_startDisclosureResult)?.type == DisclosureType.login;
  }

  /// Determines if the current session is a cross-device disclosure flow.
  bool get isCrossDeviceFlow {
    assert(_startDisclosureResult != null, '_startDisclosureResult should be set to correctly fetch isCrossDeviceFlow');
    return _startDisclosureResult?.sessionType == DisclosureSessionType.crossDevice;
  }

  DisclosureBloc(
    this._startDisclosureUseCase,
    this._cancelDisclosureUseCase,
    this._getMostRecentWalletEventUseCase,
  ) : super(const DisclosureInitial()) {
    on<DisclosureSessionStarted>(_onSessionStarted);
    on<DisclosureStopRequested>(_onStopRequested);
    on<DisclosureBackPressed>(_onBackPressed);
    on<DisclosureUrlApproved>(_onUrlApproved);
    on<DisclosureShareRequestedCardsApproved>(_onShareRequestedCardsApproved);
    on<DisclosureAlternativeCardSelected>(_onAlternativeCardSelected);
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

    /// Handle the 4 init cases:
    /// 1. Initiation errors
    /// 2. Missing attributes
    /// 3. Cross device (check url)
    /// 4. Ready to disclose (login / attributes)
    await startDisclosureResult.process(
      onError: (error) => _handleApplicationError(error, emit),
      onSuccess: (result) {
        _startDisclosureResult = result; // Cache the result;
        switch (result) {
          case StartDisclosureReadyToDisclose():
            if (isCrossDeviceFlow) {
              emit(DisclosureCheckUrl(originUrl: result.originUrl));
            } else {
              _handleReadyToDisclose(result, emit);
            }
          case StartDisclosureMissingAttributes():
            emit(
              DisclosureMissingAttributes(
                relyingParty: result.relyingParty,
                missingAttributes: result.missingAttributes,
              ),
            );
        }
      },
    );
  }

  void _handleReadyToDisclose(
    StartDisclosureReadyToDisclose result,
    Emitter<DisclosureState> emit, {
    bool afterBackPressed = false,
  }) {
    switch (result.type) {
      case DisclosureType.regular:
        emit(
          DisclosureConfirmDataAttributes(
            relyingParty: result.relyingParty,
            requestPurpose: result.requestPurpose,
            cardRequests: _cardRequestsSelectionCache ?? result.cardRequests,
            policy: result.policy,
            afterBackPressed: afterBackPressed,
            sessionType: result.sessionType,
          ),
        );
      case DisclosureType.login:
        emit(
          DisclosureCheckOrganizationForLogin(
            relyingParty: result.relyingParty,
            originUrl: result.originUrl,
            sessionType: result.sessionType,
            policy: result.policy,
            cardRequests: result.cardRequests,
            sharedDataWithOrganizationBefore: result.sharedDataWithOrganizationBefore,
            afterBackPressed: afterBackPressed,
          ),
        );
    }
  }

  Future<void> _onStopRequested(DisclosureStopRequested event, Emitter<DisclosureState> emit) async {
    emit(DisclosureLoadInProgress(state.stepperProgress));
    final relyingParty = this.relyingParty;
    final cancelResult = await _cancelDisclosureUseCase.invoke();

    // Handle the edge case where relyingParty (needed to render stopped screen) is not available.
    if (relyingParty == null) {
      await _handleApplicationError(const GenericError('relying party unavailable', sourceError: 'n/a'), emit);
      return;
    }

    await cancelResult.process(
      onSuccess: (returnUrl) {
        emit(
          DisclosureStopped(
            organization: relyingParty,
            isLoginFlow: isLoginFlow,
            returnUrl: returnUrl,
          ),
        );
      },
      onError: (error) => _handleApplicationError(error, emit),
    );
  }

  Future<void> _onBackPressed(DisclosureBackPressed event, emit) async {
    final startDisclosureResult = _startDisclosureResult;
    if (startDisclosureResult == null) return; // Unknown state, nothing to navigate back to.
    switch (state) {
      case DisclosureConfirmDataAttributes():
      case DisclosureCheckOrganizationForLogin():
        if (isCrossDeviceFlow) {
          emit(DisclosureCheckUrl(originUrl: startDisclosureResult.originUrl, afterBackPressed: true));
        }
      case DisclosureConfirmPin():
        assert(
          startDisclosureResult is StartDisclosureReadyToDisclose,
          'User should never reach $state when wallet was not ready to disclose',
        );
        _handleReadyToDisclose(startDisclosureResult as StartDisclosureReadyToDisclose, emit, afterBackPressed: true);
      default:
        assert(!state.canGoBack, 'State indicates user can navigate back, but state not updated');
        Fimber.w('State $state does not support back navigation');
    }
  }

  Future<void> _onUrlApproved(DisclosureUrlApproved event, emit) async {
    final startDisclosureResult = _startDisclosureResult;
    if (startDisclosureResult == null) throw UnsupportedError('Invalid event for state: $state');

    switch (startDisclosureResult) {
      case StartDisclosureReadyToDisclose():
        if (isLoginFlow) {
          emit(
            DisclosureCheckOrganizationForLogin(
              relyingParty: startDisclosureResult.relyingParty,
              originUrl: startDisclosureResult.originUrl,
              policy: startDisclosureResult.policy,
              cardRequests: startDisclosureResult.cardRequests,
              sessionType: startDisclosureResult.sessionType,
              sharedDataWithOrganizationBefore: startDisclosureResult.sharedDataWithOrganizationBefore,
            ),
          );
        } else {
          emit(
            DisclosureConfirmDataAttributes(
              relyingParty: startDisclosureResult.relyingParty,
              requestPurpose: startDisclosureResult.requestPurpose,
              cardRequests: _cardRequestsSelectionCache ?? startDisclosureResult.cardRequests,
              policy: startDisclosureResult.policy,
              sessionType: startDisclosureResult.sessionType,
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

  void _onShareRequestedCardsApproved(DisclosureShareRequestedCardsApproved event, emit) {
    assert(_startDisclosureResult is StartDisclosureReadyToDisclose, 'Invalid state to continue disclosing');
    assert(
      state is DisclosureConfirmDataAttributes || state is DisclosureCheckOrganizationForLogin,
      'Invalid UI state to move to pin entry',
    );
    emit(DisclosureConfirmPin(relyingParty: relyingParty!));
  }

  void _onAlternativeCardSelected(DisclosureAlternativeCardSelected event, Emitter<DisclosureState> emit) {
    final state = this.state;
    assert(state is DisclosureConfirmDataAttributes, 'BloC in invalid state for card manipulation');
    if (state is DisclosureConfirmDataAttributes) {
      final updatedState = state.updateWith(event.updatedCardRequest);
      // We store a copy of the altered user selection, so that the selection is kept when navigating away from this state.
      _cardRequestsSelectionCache = updatedState.cardRequests;
      emit(updatedState);
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

  Future<void> _onConfirmPinFailed(DisclosureConfirmPinFailed event, Emitter<DisclosureState> emit) =>
      _handleApplicationError(event.error, emit);

  Future<void> _handleApplicationError(ApplicationError error, Emitter<DisclosureState> emit) async {
    emit(DisclosureLoadInProgress(state.stepperProgress));
    switch (error) {
      case GenericError():
        emit(DisclosureGenericError(error: error, returnUrl: error.redirectUrl));
      case NetworkError():
        await _cancelDisclosureUseCase.invoke(); // Attempt to cancel the session, but propagate original error
        emit(DisclosureNetworkError(hasInternet: error.hasInternet, error: error));
      case SessionError():
        _handleSessionError(emit, error);
      case RelyingPartyError():
        emit(DisclosureRelyingPartyError(error: error, organizationName: error.organizationName));
      case ExternalScannerError():
        emit(DisclosureExternalScannerError(error: error));
      default:
        // Call cancelSession to avoid stale session and to try and provide more context (e.g. returnUrl).
        final cancelResult = await _cancelDisclosureUseCase.invoke();
        await cancelResult.process(
          onSuccess: (returnUrl) => emit(DisclosureGenericError(error: error, returnUrl: returnUrl)),
          onError: (_) => emit(DisclosureGenericError(error: error)),
        );
    }
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
          DisclosureSessionCancelled(
            error: error,
            relyingParty: relyingParty,
            returnUrl: error.returnUrl,
          ),
        );
    }
  }

  @override
  Future<void> close() async {
    await super.close();
    _startDisclosureResult = null;
    _cardRequestsSelectionCache = null;
  }
}
