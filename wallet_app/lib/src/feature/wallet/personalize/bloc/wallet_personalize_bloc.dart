import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/bloc/error_state.dart';
import '../../../../domain/model/bloc/network_error_state.dart';
import '../../../../domain/model/card/wallet_card.dart';
import '../../../../domain/model/flow_progress.dart';
import '../../../../domain/model/result/application_error.dart';
import '../../../../domain/usecase/card/get_wallet_cards_usecase.dart';
import '../../../../domain/usecase/pid/cancel_pid_issuance_usecase.dart';
import '../../../../domain/usecase/pid/continue_pid_issuance_usecase.dart';
import '../../../../domain/usecase/pid/get_pid_issuance_url_usecase.dart';
import '../../../../domain/usecase/wallet/is_wallet_initialized_with_pid_usecase.dart';
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
    );
  }

  Future<void> _continuePidIssuance(WalletPersonalizeContinuePidIssuance event, emit) async {
    emit(const WalletPersonalizeAuthenticating());
    final continueResult = await continuePidIssuanceUseCase.invoke(event.authUrl);
    await continueResult.process(
      onSuccess: (previewAttributes) => add(WalletPersonalizeLoginWithDigidSucceeded(previewAttributes)),
      onError: (error) {
        switch (error) {
          case NetworkError():
            emit(WalletPersonalizeNetworkError(error: error, hasInternet: error.hasInternet));
          case RedirectUriError():
            // Currently seeing 'accessDenied/loginRequired' when pressing cancel in the digid connector. Verify on prod. (PVW-2352)
            final cancelled = [RedirectError.accessDenied, RedirectError.loginRequired].contains(error.redirectError);
            add(WalletPersonalizeLoginWithDigidFailed(cancelledByUser: cancelled, error: error));
          case RelyingPartyError():
            emit(WalletPersonalizeRelyingPartyError(error: error, organizationName: error.organizationName));
          default:
            add(WalletPersonalizeLoginWithDigidFailed(error: error));
        }
      },
    );
  }

  Future<void> _onLoginWithDigidClicked(event, emit) async {
    emit(const WalletPersonalizeLoadingIssuanceUrl());
    // Fixes PVW-2171 (lock during WalletPersonalizeCheckData)
    await cancelPidIssuanceUseCase.invoke();

    final urlResult = await getPidIssuanceUrlUseCase.invoke();
    await urlResult.process(
      onSuccess: (url) => emit(WalletPersonalizeConnectDigid(url)),
      onError: (error) {
        switch (error) {
          case NetworkError():
            emit(WalletPersonalizeNetworkError(error: error, hasInternet: error.hasInternet));
          case RelyingPartyError():
            emit(WalletPersonalizeRelyingPartyError(error: error, organizationName: error.organizationName));
          default:
            emit(WalletPersonalizeDigidFailure(error: error));
        }
      },
    );
  }

  Future<void> _onLoginWithDigidSucceeded(WalletPersonalizeLoginWithDigidSucceeded event, emit) async {
    emit(WalletPersonalizeCheckData(availableAttributes: event.previewAttributes));
  }

  Future<void> _onLoginWithDigidFailed(WalletPersonalizeLoginWithDigidFailed event, emit) async {
    emit(WalletPersonalizeLoadInProgress(state.stepperProgress));
    await cancelPidIssuanceUseCase.invoke(); // Confirm cancellation to the server

    if (event.cancelledByUser) {
      emit(WalletPersonalizeDigidCancelled());
    } else {
      emit(WalletPersonalizeDigidFailure(error: event.error));
    }
  }

  Future<void> _onAcceptPidFailed(WalletPersonalizeAcceptPidFailed event, emit) async {
    emit(WalletPersonalizeLoadInProgress(state.stepperProgress));
    await cancelPidIssuanceUseCase.invoke(); // Confirm cancellation to the server

    final appError = event.error;
    switch (appError) {
      case NetworkError():
        emit(WalletPersonalizeNetworkError(error: appError, hasInternet: appError.hasInternet));
      case SessionError():
        emit(WalletPersonalizeSessionExpired(error: appError));
      default:
        emit(WalletPersonalizeGenericError(error: appError));
    }
  }

  Future<void> _onOfferingVerified(WalletPersonalizeOfferingAccepted event, emit) async {
    emit(WalletPersonalizeConfirmPin(event.previewAttributes));
  }

  Future<void> _onOfferingRejected(event, emit) async {
    emit(const WalletPersonalizeLoadInProgress(FlowProgress(currentStep: 0, totalSteps: kSetupSteps)));
    final cancelResult = await cancelPidIssuanceUseCase.invoke();
    if (cancelResult.hasError) Fimber.e('Failed to explicitly reject pid', ex: cancelResult.error);
    emit(const WalletPersonalizeInitial());
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
      await _loadCardsAndEmitSuccessState();
    } else {
      Fimber.e('Pin confirmed from unexpected screen');
      emit(WalletPersonalizeFailure());
    }
  }

  Future<void> _loadCardsAndEmitSuccessState() async {
    final result = await getWalletCardsUseCase.invoke();
    await result.process(
      onSuccess: (cards) {
        add(WalletPersonalizeUpdateState(WalletPersonalizeSuccess(cards)));
      },
      onError: (error) {
        Fimber.e('Failed to fetch cards', ex: error);
        add(WalletPersonalizeUpdateState(WalletPersonalizeFailure()));
      },
    );
  }
}
