import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../environment.dart';
import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/bloc/error_state.dart';
import '../../../domain/model/bloc/network_error_state.dart';
import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/model/disclosure/disclose_card_request.dart';
import '../../../domain/model/disclosure/disclosure_session_type.dart';
import '../../../domain/model/flow_progress.dart';
import '../../../domain/model/issuance/start_issuance_result.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/result/application_error.dart';
import '../../../domain/usecase/issuance/cancel_issuance_usecase.dart';
import '../../../domain/usecase/issuance/start_issuance_usecase.dart';
import '../../../util/cast_util.dart';
import '../../../util/extension/list_extension.dart';

part 'issuance_event.dart';
part 'issuance_state.dart';

class IssuanceBloc extends Bloc<IssuanceEvent, IssuanceState> {
  final StartIssuanceUseCase _startIssuanceUseCase;
  final CancelIssuanceUseCase _cancelIssuanceUseCase;

  StartIssuanceResult? _startIssuanceResult;

  Organization? get relyingParty => _startIssuanceResult?.relyingParty;

  bool get isCrossDeviceFlow {
    assert(_startIssuanceResult != null, '_startIssuanceResult should be set to correctly fetch isCrossDeviceFlow');
    return _startIssuanceResult?.sessionType == DisclosureSessionType.crossDevice;
  }

  /// A cached version of the user's card selection, used when navigating back and forth from the selection page.
  List<DiscloseCardRequest>? _cardRequestsSelectionCache;

  IssuanceBloc(
    this._startIssuanceUseCase,
    this._cancelIssuanceUseCase,
  ) : super(const IssuanceInitial()) {
    on<IssuanceSessionStarted>(_onSessionStarted);
    on<IssuanceBackPressed>(_onIssuanceBackPressed);
    on<IssuanceOrganizationApproved>(_onIssuanceOrganizationApproved);
    on<IssuanceShareRequestedAttributesDeclined>(_onIssuanceShareRequestedAttributesDeclined);
    on<IssuancePinForDisclosureConfirmed>(_onPinConfirmedForDisclosure);
    on<IssuancePinForIssuanceConfirmed>(_onPinConfirmedForIssuance);
    on<IssuanceConfirmPinFailed>(_onIssuanceConfirmPinFailed);
    on<IssuanceApproveCards>(_onCardsApproved);
    on<IssuanceCardToggled>(_onIssuanceCardToggled);
    on<IssuanceStopRequested>(_onIssuanceStopRequested);
    on<IssuanceAlternativeCardSelected>(_onAlternativeCardSelected);
  }

  Future<void> _onSessionStarted(IssuanceSessionStarted event, Emitter<IssuanceState> emit) async {
    // Cancel any potential ongoing (disclosure based) issuance session, needed for when the user taps an issuance
    // deeplink during an active issuance (or disclosure) session (e.g. by switching back to the browser).
    await _cancelIssuanceUseCase.invoke();
    final startResult = await _startIssuanceUseCase.invoke(event.issuanceUri, isQrCode: event.isQrCode);

    /// Handle [error]/[ready to disclose]/[missing attributes] cases.
    await startResult.process(
      onError: (error) => _handleApplicationError(error, emit),
      onSuccess: (result) {
        _startIssuanceResult = result;
        switch (result) {
          case StartIssuanceReadyToDisclose():
            emit(
              IssuanceCheckOrganization(
                organization: result.relyingParty,
                policy: result.policy,
                cardRequests: result.cardRequests,
                purpose: result.requestPurpose,
              ),
            );
          case StartIssuanceMissingAttributes():
            emit(
              IssuanceMissingAttributes(
                organization: result.relyingParty,
                missingAttributes: result.missingAttributes,
              ),
            );
        }
      },
    );
  }

  void _onAlternativeCardSelected(IssuanceAlternativeCardSelected event, Emitter<IssuanceState> emit) {
    final state = this.state;
    assert(state is IssuanceCheckOrganization, 'BloC in invalid state for card manipulation');
    if (state is IssuanceCheckOrganization) {
      final updatedState = state.updateWith(event.updatedCardRequest);
      // We store a copy of the altered user selection, so that the selection is kept when navigating away from this state.
      _cardRequestsSelectionCache = updatedState.cardRequests;
      emit(updatedState);
    }
  }

  Future<void> _onIssuanceBackPressed(event, Emitter<IssuanceState> emit) async {
    final state = this.state;
    final startIssuanceResult = _startIssuanceResult;
    if (startIssuanceResult == null) return; // Unknown state, nothing to navigate back to.

    switch (state) {
      case IssuanceProvidePinForDisclosure():
        assert(
          startIssuanceResult is StartIssuanceReadyToDisclose,
          'User should never reach $state when wallet was not ready to disclose',
        );
        emit(
          IssuanceCheckOrganization(
            organization: startIssuanceResult.relyingParty,
            policy: (startIssuanceResult as StartIssuanceReadyToDisclose).policy,
            cardRequests: _cardRequestsSelectionCache ?? startIssuanceResult.cardRequests,
            purpose: startIssuanceResult.requestPurpose,
            afterBackPressed: true,
          ),
        );
      case IssuanceProvidePinForIssuance():
        // Once we support selecting cards, we need to make sure we also provide the previously unselected cards here
        emit(IssuanceReviewCards.init(cards: state.cards, afterBackPressed: true));
      default:
        assert(!state.canGoBack, 'State indicates user can navigate back, but state not updated');
        Fimber.w('State $state does not support back navigation');
    }
  }

  Future<void> _onIssuanceOrganizationApproved(event, Emitter<IssuanceState> emit) async {
    final state = this.state;
    if (state is! IssuanceCheckOrganization) throw UnsupportedError('Incorrect state to $state');
    final result = _startIssuanceResult;
    if (result == null) throw UnsupportedError('Bloc in incorrect state (no data loaded)');
    switch (result) {
      case StartIssuanceReadyToDisclose():
        emit(const IssuanceProvidePinForDisclosure());
      case StartIssuanceMissingAttributes():
        emit(
          IssuanceMissingAttributes(
            organization: _startIssuanceResult!.relyingParty,
            missingAttributes: result.missingAttributes,
          ),
        );
    }
  }

  Future<void> _onIssuanceShareRequestedAttributesDeclined(event, Emitter<IssuanceState> emit) async =>
      _stopIssuance(emit);

  Future<void> _onPinConfirmedForDisclosure(
    IssuancePinForDisclosureConfirmed event,
    Emitter<IssuanceState> emit,
  ) async {
    final issuance = _startIssuanceResult;
    if (issuance == null) throw UnsupportedError('Can not move to select cards state without _startIssuanceResult');

    emit(IssuanceLoadInProgress(state.stepperProgress));
    await Future.delayed(Duration(seconds: Environment.mockRepositories ? 2 : 0));

    emit(IssuanceReviewCards.init(cards: event.cards));
  }

  Future<void> _onPinConfirmedForIssuance(IssuancePinForIssuanceConfirmed event, Emitter<IssuanceState> emit) async {
    final state = tryCast<IssuanceProvidePinForIssuance>(this.state);
    if (state == null) throw StateError('Bloc in invalid state: $state');

    emit(IssuanceLoadInProgress(state.stepperProgress));
    await Future.delayed(Duration(seconds: Environment.mockRepositories ? 2 : 0));

    emit(IssuanceCompleted(addedCards: state.cards));
  }

  void _onCardsApproved(IssuanceApproveCards event, Emitter<IssuanceState> emit) {
    if (event.cards.isEmpty) {
      _handleApplicationError(
        GenericError(
          'trying to add zero cards',
          sourceError: UnimplementedError('Card selection not yet implemented'),
        ),
        emit,
      );
    } else {
      emit(IssuanceProvidePinForIssuance(cards: event.cards));
    }
  }

  Future<void> _onIssuanceStopRequested(IssuanceStopRequested event, Emitter<IssuanceState> emit) async =>
      _stopIssuance(emit);

  FutureOr<void> _onIssuanceCardToggled(IssuanceCardToggled event, Emitter<IssuanceState> emit) {
    final state = this.state;
    if (state is! IssuanceReviewCards) throw UnsupportedError('Incorrect state to $state');
    emit(state.toggleCard(event.card));
  }

  Future<void> _stopIssuance(Emitter<IssuanceState> emit) async {
    final cancelResult = await _cancelIssuanceUseCase.invoke();
    await cancelResult.process(
      onSuccess: (returnUrl) => emit(IssuanceStopped(returnUrl: returnUrl)),
      onError: (error) => _handleApplicationError(error, emit),
    );
  }

  Future<void> _onIssuanceConfirmPinFailed(IssuanceConfirmPinFailed event, Emitter<IssuanceState> emit) =>
      _handleApplicationError(event.error, emit);

  Future<void> _handleApplicationError(ApplicationError error, Emitter<IssuanceState> emit) async {
    emit(IssuanceLoadInProgress(state.stepperProgress));
    switch (error) {
      case GenericError():
        emit(IssuanceGenericError(error: error, returnUrl: error.redirectUrl));
      case NetworkError():
        await _cancelIssuanceUseCase.invoke(); // Attempt to cancel the session, but propagate original error
        emit(IssuanceNetworkError(hasInternet: error.hasInternet, error: error));
      case SessionError():
        _handleSessionError(emit, error);
      case RelyingPartyError():
        emit(IssuanceRelyingPartyError(error: error, organizationName: error.organizationName));
      case ExternalScannerError():
        emit(IssuanceExternalScannerError(error: error));
      default:
        // Call cancelSession to avoid stale session and to try and provide more context (e.g. returnUrl).
        final cancelResult = await _cancelIssuanceUseCase.invoke();
        await cancelResult.process(
          onSuccess: (returnUrl) => emit(IssuanceGenericError(error: error, returnUrl: returnUrl)),
          onError: (_) => emit(IssuanceGenericError(error: error)),
        );
    }
  }

  void _handleSessionError(Emitter<IssuanceState> emit, SessionError error) {
    final isCrossDevice = _startIssuanceResult?.sessionType == DisclosureSessionType.crossDevice;
    switch (error.state) {
      case SessionState.expired:
        emit(
          IssuanceSessionExpired(
            error: error,
            canRetry: error.canRetry,
            isCrossDevice: isCrossDevice,
            returnUrl: error.returnUrl,
          ),
        );
      case SessionState.cancelled:
        emit(
          IssuanceSessionCancelled(
            error: error,
            relyingParty: relyingParty,
            returnUrl: error.returnUrl,
          ),
        );
    }
  }
}
