import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../environment.dart';
import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/bloc/error_state.dart';
import '../../../domain/model/bloc/network_error_state.dart';
import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/model/disclosure/disclosure_session_type.dart';
import '../../../domain/model/flow_progress.dart';
import '../../../domain/model/issuance/start_issuance_result.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/requested_attributes.dart';
import '../../../domain/model/result/application_error.dart';
import '../../../domain/usecase/issuance/cancel_issuance_usecase.dart';
import '../../../domain/usecase/issuance/start_issuance_usecase.dart';
import '../../../util/cast_util.dart';

part 'issuance_event.dart';
part 'issuance_state.dart';

class IssuanceBloc extends Bloc<IssuanceEvent, IssuanceState> {
  final StartIssuanceUseCase startIssuanceUseCase;
  final CancelIssuanceUseCase _cancelIssuanceUseCase;

  StartIssuanceResult? _startIssuanceResult;

  Organization? get relyingParty => _startIssuanceResult?.relyingParty;

  IssuanceBloc(
    this.startIssuanceUseCase,
    this._cancelIssuanceUseCase,
  ) : super(const IssuanceInitial()) {
    on<IssuanceInitiated>(_onIssuanceInitiated);
    on<IssuanceBackPressed>(_onIssuanceBackPressed);
    on<IssuanceOrganizationApproved>(_onIssuanceOrganizationApproved);
    on<IssuanceShareRequestedAttributesDeclined>(_onIssuanceShareRequestedAttributesDeclined);
    on<IssuancePinForDisclosureConfirmed>(_onPinConfirmedForDisclosure);
    on<IssuancePinForIssuanceConfirmed>(_onPinConfirmedForIssuance);
    on<IssuanceConfirmPinFailed>(_onIssuanceConfirmPinFailed);
    on<IssuanceApproveCards>(_onCardsApproved);
    on<IssuanceCardToggled>(_onIssuanceCardToggled);
    on<IssuanceStopRequested>(_onIssuanceStopRequested);
  }

  Future<void> _onIssuanceInitiated(IssuanceInitiated event, Emitter<IssuanceState> emit) async {
    // Cancel any potential ongoing (disclosure based) issuance session, needed for when the user taps an issuance
    // deeplink during an active issuance (or disclosure) session (e.g. by switching back to the browser).
    await _cancelIssuanceUseCase.invoke();

    final startResult = await startIssuanceUseCase.invoke(event.issuanceUri, isQrCode: event.isQrCode);
    _startIssuanceResult = startResult.value;
    await startResult.process(
      onSuccess: (result) {
        switch (result) {
          case StartIssuanceReadyToDisclose():
            emit(
              IssuanceCheckOrganization(
                organization: result.relyingParty,
                policy: result.policy,
                requestedAttributes: result.requestedAttributes,
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
      onError: (error) {
        switch (error) {
          case RelyingPartyError():
            emit(IssuanceRelyingPartyError(error: error, organizationName: error.organizationName));
          default:
            emit(IssuanceGenericError(error: error));
        }
      },
    );
  }

  Future<void> _onIssuanceBackPressed(event, Emitter<IssuanceState> emit) async {
    final state = this.state;
    final startIssuanceResult = _startIssuanceResult;
    if (state.canGoBack) {
      switch (state) {
        case IssuanceProvidePinForDisclosure():
          if (startIssuanceResult is StartIssuanceReadyToDisclose) {
            emit(
              IssuanceCheckOrganization(
                organization: startIssuanceResult.relyingParty,
                policy: startIssuanceResult.policy,
                requestedAttributes: startIssuanceResult.requestedAttributes,
                afterBackPressed: true,
              ),
            );
          }
        case IssuanceProvidePinForIssuance():
          // Once we support selecting cards, we need to make sure we also provide the previously unselected cards here
          emit(IssuanceReviewCards.init(cards: state.cards, afterBackPressed: true));
        default:
          return;
      }
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

    emit(IssuanceLoadInProgress(step: state.stepperProgress.currentStep));
    await Future.delayed(Duration(seconds: Environment.mockRepositories ? 2 : 0));

    emit(IssuanceReviewCards.init(cards: event.cards));
  }

  Future<void> _onPinConfirmedForIssuance(IssuancePinForIssuanceConfirmed event, Emitter<IssuanceState> emit) async {
    final state = tryCast<IssuanceProvidePinForIssuance>(this.state);
    if (state == null) throw StateError('Bloc in invalid state: $state');

    emit(IssuanceLoadInProgress(step: state.stepperProgress.currentStep));
    await Future.delayed(Duration(seconds: Environment.mockRepositories ? 2 : 0));

    emit(IssuanceCompleted(addedCards: state.cards));
  }

  Future<void> _onCardsApproved(IssuanceApproveCards event, Emitter<IssuanceState> emit) async {
    if (event.cards.isEmpty) {
      emit(
        IssuanceGenericError(
          error: GenericError(
            'Trying to add zero cards',
            sourceError: UnimplementedError('This flow is not yet implemented'),
          ),
        ),
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

  Future<void> _stopIssuance(Emitter<IssuanceState> emit) async {
    final cancelResult = await _cancelIssuanceUseCase.invoke();
    await cancelResult.process(
      onSuccess: (returnUrl) => emit(IssuanceStopped(returnUrl: returnUrl)),
      onError: (error) => emit(IssuanceGenericError(error: error)),
    );
  }

  Future<void> _onIssuanceConfirmPinFailed(IssuanceConfirmPinFailed event, Emitter<IssuanceState> emit) async {
    emit(IssuanceLoadInProgress(step: state.stepperProgress.currentStep));
    // {event.error} is the error that was thrown when user tried to confirm the disclosure/issuance session with a PIN.
    final error = event.error;
    switch (error) {
      case SessionError():
        _handleSessionError(emit, error);
        return;
      case RelyingPartyError():
        emit(IssuanceRelyingPartyError(error: error, organizationName: error.organizationName));
        return;
      case NetworkError():
        await _cancelIssuanceUseCase.invoke(); // Attempt to cancel the session, but propagate original error
        emit(IssuanceNetworkError(error: error, hasInternet: error.hasInternet));
        return;
      default:
        Fimber.d('Disclosure failed unexpectedly when entering pin, cause: ${event.error}.');
    }
    // Call cancelSession to avoid stale session and to try and provide more context (e.g. returnUrl).
    final cancelResult = await _cancelIssuanceUseCase.invoke();
    await cancelResult.process(
      onSuccess: (returnUrl) => emit(IssuanceGenericError(error: event.error, returnUrl: returnUrl)),
      onError: (error) => emit(IssuanceGenericError(error: error)),
    );
  }
}
