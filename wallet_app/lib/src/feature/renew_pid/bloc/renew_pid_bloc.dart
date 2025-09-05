import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/bloc/error_state.dart';
import '../../../domain/model/bloc/network_error_state.dart';
import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/model/flow_progress.dart';
import '../../../domain/model/result/application_error.dart';
import '../../../domain/usecase/card/get_pid_cards_usecase.dart';
import '../../../domain/usecase/pid/cancel_pid_issuance_usecase.dart';
import '../../../domain/usecase/pid/continue_pid_issuance_usecase.dart';
import '../../../domain/usecase/pid/get_pid_renewal_url_usecase.dart';
import '../../../wallet_core/error/core_error.dart';

part 'renew_pid_event.dart';
part 'renew_pid_state.dart';

class RenewPidBloc extends Bloc<RenewPidEvent, RenewPidState> {
  final GetPidRenewalUrlUseCase _getPidRenewalUrlUseCase;
  final ContinuePidIssuanceUseCase _continuePidIssuanceUseCase;
  final CancelPidIssuanceUseCase _cancelPidIssuanceUseCase;
  final GetPidCardsUseCase _getWalletCardsUseCase;

  RenewPidBloc(
    this._getPidRenewalUrlUseCase,
    this._continuePidIssuanceUseCase,
    this._cancelPidIssuanceUseCase,
    this._getWalletCardsUseCase, {
    required bool continueFromDigiD,
  }) : super(continueFromDigiD ? const RenewPidVerifyingDigidAuthentication() : const RenewPidInitial()) {
    on<RenewPidLoginWithDigidClicked>(_onDigidLoginClicked);
    on<RenewPidLoginWithDigidFailed>(_onDigidLoginFailed);
    on<RenewPidContinuePidRenewal>(_onContinuePidRenewal);
    on<RenewPidAttributesConfirmed>(_onAttributesConfirmed);
    on<RenewPidAttributesRejected>(_onAttributesRejected);
    on<RenewPidPinConfirmed>(_onPinConfirmed);
    on<RenewPidPinConfirmationFailed>(_onPinConfirmationFailed);
    on<RenewPidBackPressed>(_onBackPressed);
    on<RenewPidRetryPressed>(_onRetryPressed);
    on<RenewPidStopPressed>(_onStopPressed);
  }

  FutureOr<void> _onDigidLoginClicked(RenewPidLoginWithDigidClicked event, Emitter<RenewPidState> emit) async {
    emit(const RenewPidLoadingDigidUrl());
    unawaited(_cancelPidIssuanceUseCase.invoke()); // Cancel any potential stale session
    final result = await _getPidRenewalUrlUseCase.invoke();
    await result.process(
      onSuccess: (url) => emit(RenewPidAwaitingDigidAuthentication(url)),
      onError: (error) => _handleApplicationError(error, emit),
    );
  }

  FutureOr<void> _onContinuePidRenewal(RenewPidContinuePidRenewal event, Emitter<RenewPidState> emit) async {
    emit(const RenewPidVerifyingDigidAuthentication());
    final result = await _continuePidIssuanceUseCase.invoke(event.authUrl);
    await result.process(
      onSuccess: (attributes) => emit(RenewPidCheckData(availableAttributes: attributes)),
      onError: (error) => _handleApplicationError(error, emit),
    );
  }

  Future<void> _handleApplicationError(ApplicationError error, Emitter<RenewPidState> emit) async {
    await _cancelPidIssuanceUseCase.invoke(); // Always attempt to cancel the session, then render the specific error

    // TODO(Rob): Handle DigiDMismatch and emit [RenewPidDigidMismatch]
    switch (error) {
      case NetworkError():
        emit(RenewPidNetworkError(hasInternet: error.hasInternet, error: error));
      case RedirectUriError():
        if ([RedirectError.accessDenied, RedirectError.loginRequired].contains(error.redirectError)) {
          emit(const RenewPidDigidLoginCancelled());
        } else {
          emit(RenewPidGenericError(error: error));
        }
      default:
        Fimber.w('Handling ${error.runtimeType} as generic error.', ex: error);
        emit(RenewPidGenericError(error: error));
    }
  }

  FutureOr<void> _onAttributesConfirmed(RenewPidAttributesConfirmed event, Emitter<RenewPidState> emit) {
    emit(RenewPidConfirmPin(event.previewAttributes));
  }

  FutureOr<void> _onAttributesRejected(RenewPidAttributesRejected event, Emitter<RenewPidState> emit) {
    emit(const RenewPidInitial(didGoBack: true));
  }

  FutureOr<void> _onPinConfirmed(RenewPidPinConfirmed event, Emitter<RenewPidState> emit) async {
    emit(const RenewPidUpdatingCards());
    final result = await _getWalletCardsUseCase.invoke();
    await result.process(
      onSuccess: (cards) => emit(RenewPidSuccess(cards)),
      onError: (error) => _handleApplicationError(error, emit),
    );
  }

  FutureOr<void> _onPinConfirmationFailed(RenewPidPinConfirmationFailed event, Emitter<RenewPidState> emit) async {
    await _handleApplicationError(event.error, emit);
  }

  FutureOr<void> _onBackPressed(RenewPidBackPressed event, Emitter<RenewPidState> emit) async {
    final state = this.state;
    if (!state.canGoBack) return;
    if (state is RenewPidConfirmPin) emit(RenewPidCheckData(availableAttributes: state.attributes, didGoBack: true));
  }

  FutureOr<void> _onDigidLoginFailed(RenewPidLoginWithDigidFailed event, Emitter<RenewPidState> emit) async {
    if (event.cancelledByUser) {
      emit(const RenewPidDigidLoginCancelled());
    } else {
      await _handleApplicationError(event.error, emit);
    }
  }

  FutureOr<void> _onRetryPressed(RenewPidRetryPressed event, Emitter<RenewPidState> emit) {
    add(const RenewPidLoginWithDigidClicked());
  }

  FutureOr<void> _onStopPressed(RenewPidStopPressed event, Emitter<RenewPidState> emit) async {
    unawaited(_cancelPidIssuanceUseCase.invoke());
    emit(const RenewPidStopped());
  }
}
