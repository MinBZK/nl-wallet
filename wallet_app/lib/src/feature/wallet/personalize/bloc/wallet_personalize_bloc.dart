import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../data/repository/pid/pid_repository.dart';
import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/bloc/error_state.dart';
import '../../../../domain/model/bloc/network_error_state.dart';
import '../../../../domain/model/flow_progress.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../../domain/usecase/card/get_wallet_cards_usecase.dart';
import '../../../../domain/usecase/pid/cancel_pid_issuance_usecase.dart';
import '../../../../domain/usecase/pid/continue_pid_issuance_usecase.dart';
import '../../../../domain/usecase/pid/get_pid_issuance_url_usecase.dart';
import '../../../../domain/usecase/wallet/is_wallet_initialized_with_pid_usecase.dart';
import '../../../../util/extension/bloc_extension.dart';
import '../../../../wallet_constants.dart';
import '../../../../wallet_core/error/core_error.dart';

part 'wallet_personalize_event.dart';
part 'wallet_personalize_state.dart';

class WalletPersonalizeBloc extends Bloc<WalletPersonalizeEvent, WalletPersonalizeState> {
  final GetWalletCardsUseCase getWalletCardsUseCase;
  final GetPidIssuanceUrlUseCase getPidIssuanceUrlUseCase;
  final CancelPidIssuanceUseCase cancelPidIssuanceUseCase;
  final ContinuePidIssuanceUseCase continuePidIssuanceUseCase;
  final IsWalletInitializedWithPidUseCase isWalletInitializedWithPidUseCase;

  WalletPersonalizeBloc(
    this.getWalletCardsUseCase,
    this.getPidIssuanceUrlUseCase,
    this.cancelPidIssuanceUseCase,
    this.continuePidIssuanceUseCase,
    this.isWalletInitializedWithPidUseCase, {
    bool continueFromDigiD = false,
  }) : super(continueFromDigiD ? const WalletPersonalizeAuthenticating() : const WalletPersonalizeInitial()) {
    on<WalletPersonalizeLoginWithDigidClicked>(_onLoginWithDigidClicked);
    on<WalletPersonalizeLoginWithDigidSucceeded>(_onLoginWithDigidSucceeded);
    on<WalletPersonalizeLoginWithDigidFailed>(_onLoginWithDigidFailed);
    on<WalletPersonalizeAcceptPidFailed>(_onAcceptPidFailed);
    on<WalletPersonalizeOfferingAccepted>(_onOfferingVerified);
    on<WalletPersonalizeOfferingRejected>(_onOfferingRejected);
    on<WalletPersonalizePinConfirmed>(_onPinConfirmed);
    on<WalletPersonalizeBackPressed>(_onBackPressed);
    on<WalletPersonalizeRetryPressed>(_onRetryPressed);
    on<WalletPersonalizeUpdateState>(_onStateUpdate);
    on<WalletPersonalizeContinuePidIssuance>(_continuePidIssuance);

    // PID sanity check (PVW-2742)
    isWalletInitializedWithPidUseCase.invoke().then(
      (initialized) async {
        if (initialized) await _loadCardsAndEmitSuccessState();
      },
      onError: (_) {}, /* consume any errors */
    );
  }

  Future<void> _continuePidIssuance(WalletPersonalizeContinuePidIssuance event, emit) async {
    try {
      add(const WalletPersonalizeUpdateState(WalletPersonalizeAuthenticating()));
      final result = await continuePidIssuanceUseCase.invoke(event.authUrl);
      switch (result) {
        case PidIssuanceSuccess():
          add(WalletPersonalizeLoginWithDigidSucceeded(result.previews));
        case PidIssuanceError():

          /// Currently seeing 'accessDenied' when pressing cancel in the digid connector. Verify on prod. (PVW-2352)
          final cancelledByUser = result.error == RedirectError.accessDenied;
          add(WalletPersonalizeLoginWithDigidFailed(cancelledByUser: cancelledByUser, error: result.error));
      }
    } catch (ex) {
      await handleError(
        ex,
        onNetworkError: (ex, hasInternet) => add(
          WalletPersonalizeUpdateState(
            WalletPersonalizeNetworkError(error: ex, hasInternet: hasInternet),
          ),
        ),
        onCoreExpiredSessionError: (ex) => emit(WalletPersonalizeSessionExpired(error: ex)),
        onUnhandledError: (ex) => add(WalletPersonalizeLoginWithDigidFailed(error: ex)),
      );
    }
  }

  Future<void> _onLoginWithDigidClicked(event, emit) async {
    try {
      emit(const WalletPersonalizeLoadingIssuanceUrl());
      // Fixes PVW-2171 (lock during WalletPersonalizeCheckData)
      await cancelPidIssuanceUseCase.invoke();
      final String url = await getPidIssuanceUrlUseCase.invoke();
      emit(WalletPersonalizeConnectDigid(url));
    } catch (ex, stack) {
      Fimber.e('Failed to get authentication url', ex: ex, stacktrace: stack);
      await handleError(
        ex,
        onNetworkError: (ex, hasInternet) => emit(WalletPersonalizeNetworkError(error: ex, hasInternet: hasInternet)),
        onUnhandledError: (ex) => emit(WalletPersonalizeDigidFailure(error: ex)),
      );
    }
  }

  Future<void> _onLoginWithDigidSucceeded(WalletPersonalizeLoginWithDigidSucceeded event, emit) async {
    emit(WalletPersonalizeCheckData(availableAttributes: event.previewAttributes));
  }

  Future<void> _onLoginWithDigidFailed(WalletPersonalizeLoginWithDigidFailed event, emit) async {
    Object error = event.error ?? 'unknown';
    try {
      emit(WalletPersonalizeLoadInProgress(state.stepperProgress));
      await cancelPidIssuanceUseCase.invoke();
    } catch (cancellationError) {
      Fimber.e('Failed to cancel PID issuance', ex: cancellationError);
      // Prefer exposing the original event error if it exists, otherwise expose the cancellation error.
      error = event.error ?? cancellationError;
    } finally {
      if (event.cancelledByUser) {
        emit(WalletPersonalizeDigidCancelled());
      } else {
        emit(WalletPersonalizeDigidFailure(error: error));
      }
    }
  }

  Future<void> _onAcceptPidFailed(WalletPersonalizeAcceptPidFailed event, emit) async {
    Object error = event.error ?? 'unknown';
    try {
      emit(WalletPersonalizeLoadInProgress(state.stepperProgress));
      await cancelPidIssuanceUseCase.invoke();
    } catch (cancellationError) {
      Fimber.e('Failed to cancel pid issuance', ex: cancellationError);
      // Prefer exposing the original event error if it exists, otherwise expose the cancellation error.
      error = event.error ?? cancellationError;
    } finally {
      await handleError(
        error,
        onNetworkError: (ex, hasInternet) => emit(WalletPersonalizeNetworkError(error: ex, hasInternet: hasInternet)),
        onCoreExpiredSessionError: (ex) => emit(WalletPersonalizeSessionExpired(error: ex)),
        onUnhandledError: (ex) => emit(WalletPersonalizeGenericError(error: ex)),
      );
    }
  }

  Future<void> _onOfferingVerified(WalletPersonalizeOfferingAccepted event, emit) async {
    emit(WalletPersonalizeConfirmPin(event.previewAttributes));
  }

  Future<void> _onOfferingRejected(event, emit) async {
    emit(const WalletPersonalizeLoadInProgress(FlowProgress(currentStep: 0, totalSteps: kSetupSteps)));
    try {
      await cancelPidIssuanceUseCase.invoke();
    } catch (ex) {
      Fimber.e('Failed to explicitly reject pid', ex: ex);
    } finally {
      emit(const WalletPersonalizeInitial());
    }
  }

  Future<void> _onRetryPressed(event, emit) async => emit(const WalletPersonalizeInitial());

  void _onStateUpdate(WalletPersonalizeUpdateState event, emit) => emit(event.state);

  Future<void> _onBackPressed(event, emit) async {
    final state = this.state;
    if (state.canGoBack) {
      if (state is WalletPersonalizeConfirmPin) {
        emit(WalletPersonalizeCheckData(didGoBack: true, availableAttributes: state.attributes));
      }
    }
  }

  Future<void> _onPinConfirmed(event, emit) async {
    final state = this.state;
    if (state is WalletPersonalizeConfirmPin) {
      emit(WalletPersonalizeAddingCards(state.stepperProgress));
      try {
        await _loadCardsAndEmitSuccessState();
      } catch (ex, stack) {
        Fimber.e('Failed to add cards to wallet', ex: ex, stacktrace: stack);
        emit(WalletPersonalizeFailure());
      }
    }
  }

  Future<void> _loadCardsAndEmitSuccessState() async {
    try {
      final cards = await getWalletCardsUseCase.invoke();
      add(WalletPersonalizeUpdateState(WalletPersonalizeSuccess(cards)));
    } catch (ex, stack) {
      Fimber.e('Failed to fetch cards from wallet', ex: ex, stacktrace: stack);
      add(WalletPersonalizeUpdateState(WalletPersonalizeFailure()));
    }
  }
}
