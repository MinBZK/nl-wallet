import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
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
import '../../../wallet_core/error/core_error.dart';
import '../../report_issue/report_issue_screen.dart';

part 'disclosure_event.dart';
part 'disclosure_state.dart';

class DisclosureBloc extends Bloc<DisclosureEvent, DisclosureState> {
  final StartDisclosureUseCase _startDisclosureUseCase;
  final CancelDisclosureUseCase _cancelDisclosureUseCase;

  StartDisclosureResult? _startDisclosureResult;

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

  Future<void> _onSessionStarted(DisclosureSessionStarted event, Emitter<DisclosureState> emit) async {
    try {
      // Cancel any potential ongoing disclosure, this can happen when a second disclosure
      // deeplink is pressed while the disclosure flow is currently open. This opens a second
      // disclosure bloc before the original one is closed, thus we need to cancel it here.
      await _cancelDisclosureUseCase.invoke();
      final startDisclosureResult =
          _startDisclosureResult = await _startDisclosureUseCase.invoke(event.uri, isQrCode: event.isQrCode);
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
        onDisclosureSourceMismatchError: (error) {
          if (error.isCrossDevice) {
            emit(DisclosureExternalScannerError(error: error));
          } else {
            emit(DisclosureGenericError(error: error, returnUrl: error.returnUrl));
          }
        },
        onCoreExpiredSessionError: (error) => emit(
          DisclosureSessionExpired(
            error: error,
            canRetry: error.canRetry,
            isCrossDevice: event.isQrCode,
            returnUrl: error.returnUrl,
          ),
        ),
        onCoreError: (error) => emit(DisclosureGenericError(error: error, returnUrl: error.returnUrl)),
        onUnhandledError: (error) => emit(DisclosureGenericError(error: error)),
      );
    }
  }

  Future<void> _onStopRequested(DisclosureStopRequested event, emit) async {
    String? returnUrl;
    try {
      emit(DisclosureLoadInProgress());
      returnUrl = await _cancelDisclosureUseCase.invoke();
    } catch (ex) {
      Fimber.e('Failed to explicitly cancel disclosure flow', ex: ex);
      returnUrl = tryCast<CoreError>(ex)?.returnUrl;
    } finally {
      final relyingParty = _startDisclosureResult?.relyingParty;
      if (relyingParty == null) {
        emit(
          DisclosureGenericError(
            error: 'Invalid state, no relying party to render stopped',
            returnUrl: returnUrl,
          ),
        );
      } else {
        emit(
          DisclosureStopped(
            organization: _startDisclosureResult!.relyingParty,
            isLoginFlow: isLoginFlow,
            returnUrl: returnUrl,
          ),
        );
      }
    }
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

  Future<void> _onReportPressed(DisclosureReportPressed event, Emitter<DisclosureState> emit) async {
    Fimber.d('User selected reporting option ${event.option}');
    String? returnUrl;
    try {
      emit(DisclosureLoadInProgress());
      returnUrl = await _cancelDisclosureUseCase.invoke();
    } catch (ex) {
      Fimber.e('Failed to explicitly cancel disclosure flow', ex: ex);
      returnUrl = tryCast<CoreError>(ex)?.returnUrl;
    } finally {
      emit(DisclosureLeftFeedback(returnUrl: returnUrl));
    }
  }

  Future<void> _onConfirmPinFailed(DisclosureConfirmPinFailed event, Emitter<DisclosureState> emit) async {
    String? returnUrl;
    try {
      emit(DisclosureLoadInProgress());
      Fimber.d('Disclosure failed when entering pin, cause: ${event.error}. Calling cancelSession explicitly as well.');
      returnUrl = await _cancelDisclosureUseCase.invoke();
    } catch (ex) {
      Fimber.e('Failed to explicitly cancel disclosure session', ex: ex);
      returnUrl = tryCast<CoreError>(ex)?.returnUrl;
    } finally {
      await handleError(
        event.error,
        onNetworkError: (error, hasInternet) => emit(DisclosureNetworkError(error: error, hasInternet: hasInternet)),
        onCoreExpiredSessionError: (error) {
          final isCrossDevice = _startDisclosureResult?.sessionType == DisclosureSessionType.crossDevice;
          emit(
            DisclosureSessionExpired(
              error: error,
              canRetry: error.canRetry,
              isCrossDevice: isCrossDevice,
              returnUrl: error.returnUrl ?? returnUrl,
            ),
          );
        },
        onCoreCancelledSessionError: (error) => emit(
          DisclosureCancelledSessionError(
            error: error,
            relyingParty: relyingParty!,
            returnUrl: error.returnUrl ?? returnUrl,
          ),
        ),
        onCoreError: (error) => emit(DisclosureGenericError(error: error, returnUrl: error.returnUrl ?? returnUrl)),
        onUnhandledError: (error) => emit(DisclosureGenericError(error: error, returnUrl: returnUrl)),
      );
    }
  }
}

extension _CoreErrorExtension on CoreError {
  String? get returnUrl => data?['return_url'];
}
