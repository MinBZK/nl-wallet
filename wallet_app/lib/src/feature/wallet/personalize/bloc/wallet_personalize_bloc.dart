import 'dart:async';

import 'package:collection/collection.dart';
import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../data/repository/pid/pid_repository.dart';
import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../../domain/usecase/card/get_pid_issuance_response_usecase.dart';
import '../../../../domain/usecase/card/get_wallet_cards_usecase.dart';
import '../../../../domain/usecase/card/wallet_add_issued_cards_usecase.dart';
import '../../../../domain/usecase/pid/get_pid_issuance_url_usecase.dart';
import '../../../../domain/usecase/pid/observe_pid_issuance_status_usecase.dart';
import '../../../../util/extension/bloc_extension.dart';
import '../../../../wallet_constants.dart';
import '../../../../wallet_core/error/core_error.dart';

part 'wallet_personalize_event.dart';
part 'wallet_personalize_state.dart';

class WalletPersonalizeBloc extends Bloc<WalletPersonalizeEvent, WalletPersonalizeState> {
  final GetPidIssuanceResponseUseCase getPidIssuanceResponseUseCase;
  final WalletAddIssuedCardsUseCase walletAddIssuedCardsUseCase;
  final GetWalletCardsUseCase getWalletCardsUseCase;
  final GetPidIssuanceUrlUseCase getPidIssuanceUrlUseCase;
  final ObservePidIssuanceStatusUseCase observePidIssuanceStatusUseCase;

  StreamSubscription? _pidIssuanceStatusSubscription;

  WalletPersonalizeBloc(
    this.getPidIssuanceResponseUseCase,
    this.walletAddIssuedCardsUseCase,
    this.getWalletCardsUseCase,
    this.getPidIssuanceUrlUseCase,
    this.observePidIssuanceStatusUseCase,
  ) : super(const WalletPersonalizeInitial()) {
    on<WalletPersonalizeLoginWithDigidClicked>(_onLoginWithDigidClicked);
    on<WalletPersonalizeLoginWithDigidSucceeded>(_onLoginWithDigidSucceeded);
    on<WalletPersonalizeLoginWithDigidFailed>(_onLoginWithDigidFailed);
    on<WalletPersonalizeOfferingVerified>(_onOfferingVerified);
    on<WalletPersonalizePinConfirmed>(_onPinConfirmed);
    on<WalletPersonalizeOnBackPressed>(_onBackPressed);
    on<WalletPersonalizeOnRetryClicked>(_onRetryClicked);
    on<WalletPersonalizeAuthInProgress>(_onAuthInProgress);

    _pidIssuanceStatusSubscription = observePidIssuanceStatusUseCase.invoke().listen(_handlePidIssuanceStatusUpdate);
  }

  void _handlePidIssuanceStatusUpdate(PidIssuanceStatus event) {
    if (state is WalletPersonalizeDigidFailure) return; // Don't navigate when user cancelled.
    switch (event) {
      case PidIssuanceIdle():
        break;
      case PidIssuanceAuthenticating():
        add(WalletPersonalizeAuthInProgress());
        break;
      case PidIssuanceSuccess():
        add(WalletPersonalizeLoginWithDigidSucceeded(event.previews));
        break;
      case PidIssuanceError():
        //TODO: Currently seeing 'accessDenied' when pressing cancel in the digid connector. To be verified on PROD.
        final cancelledByUser = event.error == RedirectError.accessDenied;
        add(WalletPersonalizeLoginWithDigidFailed(cancelledByUser: cancelledByUser));
        break;
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
      emit(WalletPersonalizeDigidCancelled());
    } else {
      emit(WalletPersonalizeDigidFailure());
    }
  }

  void _onOfferingVerified(WalletPersonalizeOfferingVerified event, emit) async {
    emit(const WalletPersonalizeConfirmPin());
  }

  void _onRetryClicked(event, emit) async => emit(const WalletPersonalizeInitial());

  void _onAuthInProgress(event, emit) async => emit(const WalletPersonalizeAuthenticating());

  void _onBackPressed(event, emit) async {
    final state = this.state;
    if (state.canGoBack) {
      if (state is WalletPersonalizeConfirmPin) {
        final issuanceResponse = await getPidIssuanceResponseUseCase.invoke();
        final allAttributes = issuanceResponse.cards.map((e) => e.attributes).flattened;
        emit(
          WalletPersonalizeCheckData(
            didGoBack: true,
            availableAttributes: allAttributes.toList(),
          ),
        );
      }
    }
  }

  Future<void> _onPinConfirmed(event, emit) async {
    final state = this.state;
    if (state is WalletPersonalizeConfirmPin) {
      emit(const WalletPersonalizeLoadInProgress(5));
      await Future.delayed(kDefaultMockDelay);
      try {
        final issuanceResponse = await getPidIssuanceResponseUseCase.invoke();
        await walletAddIssuedCardsUseCase.invoke(issuanceResponse.cards, issuanceResponse.organization);
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

  @override
  Future<void> close() async {
    _pidIssuanceStatusSubscription?.cancel();
    super.close();
  }
}
