import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../data/repository/pid/pid_repository.dart';
import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../../domain/usecase/card/get_wallet_cards_usecase.dart';
import '../../../../domain/usecase/pid/cancel_pid_issuance_usecase.dart';
import '../../../../domain/usecase/pid/continue_pid_issuance_usecase.dart';
import '../../../../domain/usecase/pid/get_pid_issuance_url_usecase.dart';
import '../../../../domain/usecase/pid/reject_offered_pid_usecase.dart';
import '../../../../util/extension/bloc_extension.dart';
import '../../../../wallet_core/error/core_error.dart';

part 'wallet_personalize_event.dart';
part 'wallet_personalize_state.dart';

class WalletPersonalizeBloc extends Bloc<WalletPersonalizeEvent, WalletPersonalizeState> {
  final GetWalletCardsUseCase getWalletCardsUseCase;
  final GetPidIssuanceUrlUseCase getPidIssuanceUrlUseCase;
  final CancelPidIssuanceUseCase cancelPidIssuanceUseCase;
  final RejectOfferedPidUseCase rejectOfferedPidUseCase;
  final ContinuePidIssuanceUseCase continuePidIssuanceUseCase;

  WalletPersonalizeBloc(
    Uri? pidIssuanceUri,
    this.getWalletCardsUseCase,
    this.getPidIssuanceUrlUseCase,
    this.cancelPidIssuanceUseCase,
    this.rejectOfferedPidUseCase,
    this.continuePidIssuanceUseCase,
  ) : super(pidIssuanceUri == null ? const WalletPersonalizeInitial() : const WalletPersonalizeAuthenticating()) {
    on<WalletPersonalizeLoginWithDigidClicked>(_onLoginWithDigidClicked);
    on<WalletPersonalizeLoginWithDigidSucceeded>(_onLoginWithDigidSucceeded);
    on<WalletPersonalizeLoginWithDigidFailed>(_onLoginWithDigidFailed);
    on<WalletPersonalizeOfferingAccepted>(_onOfferingVerified);
    on<WalletPersonalizeOfferingRejected>(_onOfferingRejected);
    on<WalletPersonalizePinConfirmed>(_onPinConfirmed);
    on<WalletPersonalizeOnBackPressed>(_onBackPressed);
    on<WalletPersonalizeOnRetryClicked>(_onRetryClicked);
    on<WalletPersonalizeAuthInProgress>(_onAuthInProgress);

    if (pidIssuanceUri != null) {
      _continuePidIssuance(pidIssuanceUri);
    }
  }

  void _continuePidIssuance(Uri uri) async {
    try {
      add(WalletPersonalizeAuthInProgress());
      final result = await continuePidIssuanceUseCase.invoke(uri);
      switch (result) {
        case PidIssuanceSuccess():
          add(WalletPersonalizeLoginWithDigidSucceeded(result.previews));
        case PidIssuanceError():
          //TODO: Currently seeing 'accessDenied' when pressing cancel in the digid connector. To be verified on PROD.
          final cancelledByUser = result.error == RedirectError.accessDenied;
          add(WalletPersonalizeLoginWithDigidFailed(cancelledByUser: cancelledByUser));
      }
    } catch (ex) {
      add(const WalletPersonalizeLoginWithDigidFailed());
    }
  }

  void _onLoginWithDigidClicked(event, emit) async {
    try {
      emit(const WalletPersonalizeLoadingIssuanceUrl());
      String url = await getPidIssuanceUrlUseCase.invoke();
      emit(WalletPersonalizeConnectDigid(url));
    } catch (ex, stack) {
      Fimber.e('Failed to get authentication url', ex: ex, stacktrace: stack);
      handleError(
        ex,
        onUnhandledError: (ex) => emit(WalletPersonalizeDigidFailure()),
      );
    }
  }

  void _onLoginWithDigidSucceeded(WalletPersonalizeLoginWithDigidSucceeded event, emit) async {
    emit(WalletPersonalizeCheckData(availableAttributes: event.previewAttributes));
  }

  void _onLoginWithDigidFailed(WalletPersonalizeLoginWithDigidFailed event, emit) async {
    if (event.cancelledByUser) {
      try {
        await cancelPidIssuanceUseCase.invoke();
      } catch (ex, stack) {
        Fimber.e('Failed to cancel PID issuance', ex: ex, stacktrace: stack);
      } finally {
        emit(WalletPersonalizeDigidCancelled());
      }
    } else {
      emit(WalletPersonalizeDigidFailure());
    }
  }

  void _onOfferingVerified(WalletPersonalizeOfferingAccepted event, emit) async {
    emit(WalletPersonalizeConfirmPin(event.previewAttributes));
  }

  void _onOfferingRejected(event, emit) async {
    emit(const WalletPersonalizeLoadInProgress(0));
    try {
      await rejectOfferedPidUseCase.invoke();
    } catch (ex) {
      Fimber.e('Failed to explicitly reject pid', ex: ex);
    } finally {
      emit(const WalletPersonalizeInitial());
    }
  }

  void _onRetryClicked(event, emit) async => emit(const WalletPersonalizeInitial());

  void _onAuthInProgress(event, emit) async => emit(const WalletPersonalizeAuthenticating());

  void _onBackPressed(event, emit) async {
    final state = this.state;
    if (state.canGoBack) {
      if (state is WalletPersonalizeConfirmPin) {
        emit(
          WalletPersonalizeCheckData(didGoBack: true, availableAttributes: state.attributes),
        );
      }
    }
  }

  Future<void> _onPinConfirmed(event, emit) async {
    final state = this.state;
    if (state is WalletPersonalizeConfirmPin) {
      emit(const WalletPersonalizeLoadInProgress(0.96));
      try {
        await _loadCardsAndEmitSuccessState(event, emit);
      } catch (ex, stack) {
        Fimber.e('Failed to add cards to wallet', ex: ex, stacktrace: stack);
        emit(WalletPersonalizeFailure());
      }
    }
  }

  Future<void> _loadCardsAndEmitSuccessState(event, emit) async {
    try {
      final cards = await getWalletCardsUseCase.invoke();
      emit(WalletPersonalizeSuccess(cards));
    } catch (ex, stack) {
      Fimber.e('Failed to fetch cards from wallet', ex: ex, stacktrace: stack);
      emit(WalletPersonalizeFailure());
    }
  }
}
