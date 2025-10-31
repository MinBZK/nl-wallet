import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/service/auto_lock_service.dart';
import '../../../domain/model/bloc/error_state.dart';
import '../../../domain/model/bloc/network_error_state.dart';
import '../../../domain/model/flow_progress.dart';
import '../../../domain/model/result/application_error.dart';
import '../../../domain/model/transfer/transfer_session_state.dart';
import '../../../domain/usecase/transfer/cancel_wallet_transfer_usecase.dart';
import '../../../domain/usecase/transfer/init_wallet_transfer_usecase.dart';
import '../../../domain/usecase/transfer/observe_transfer_session_state_usecase.dart';
import '../../../domain/usecase/transfer/receive_wallet_transfer_usecase.dart';
import '../../../domain/usecase/transfer/skip_wallet_transfer_usecase.dart';
import '../../../util/cast_util.dart';

part 'wallet_transfer_target_event.dart';
part 'wallet_transfer_target_state.dart';

class WalletTransferTargetBloc extends Bloc<WalletTransferTargetEvent, WalletTransferTargetState> {
  final InitWalletTransferUseCase _initWalletTransferUseCase;
  final ObserveTransferSessionStateUseCase _observeTransferSessionStateUseCase;
  final CancelWalletTransferUseCase _cancelWalletTransferUsecase;
  final SkipWalletTransferUseCase _skipWalletTransferUseCase;
  final ReceiveWalletTransferUseCase _receiveWalletTransferUseCase;
  final AutoLockService _autoLockProvider;

  StreamSubscription? _sessionStateSubscription;

  WalletTransferTargetBloc(
    this._initWalletTransferUseCase,
    this._observeTransferSessionStateUseCase,
    this._cancelWalletTransferUsecase,
    this._skipWalletTransferUseCase,
    this._receiveWalletTransferUseCase,
    this._autoLockProvider,
  ) : super(const WalletTransferIntroduction()) {
    on<WalletTransferRestartEvent>(_onUserRestart);
    on<WalletTransferOptInEvent>(_onUserOptIn);
    on<WalletTransferOptOutEvent>(_onUserOptOut);
    on<WalletTransferStopRequestedEvent>(_onStopRequested);
    on<WalletTransferBackPressedEvent>(_onBackPressed);
    on<WalletTransferUpdateStateEvent>((event, emit) => emit(event.state));
  }

  @override
  void onChange(Change<WalletTransferTargetState> change) {
    super.onChange(change);
    switch (change.nextState) {
      case WalletTransferLoadingQrData():
      case WalletTransferAwaitingQrScan():
      case WalletTransferAwaitingConfirmation():
      case WalletTransferTransferring():
        _autoLockProvider.setAutoLock(enabled: false);
      case WalletTransferIntroduction():
      case WalletTransferSuccess():
      case WalletTransferStopped():
      case WalletTransferGenericError():
      case WalletTransferNetworkError():
      case WalletTransferSessionExpired():
      case WalletTransferFailed():
        _autoLockProvider.setAutoLock(enabled: true);
    }
  }

  FutureOr<void> _onUserRestart(WalletTransferRestartEvent event, Emitter<WalletTransferTargetState> emit) {
    unawaited(_sessionStateSubscription?.cancel());
    emit(const WalletTransferIntroduction(didGoBack: true));
  }

  Future<void> _onUserOptIn(
    WalletTransferOptInEvent event,
    Emitter<WalletTransferTargetState> emit,
  ) async {
    emit(const WalletTransferLoadingQrData());
    final result = await _initWalletTransferUseCase.invoke();
    await result.process(
      onSuccess: (qrData) {
        if (state is! WalletTransferLoadingQrData) return; // User cancelled
        emit(WalletTransferAwaitingQrScan(qrData));
        _startObservingSessionState(qrData);
      },
      onError: (ApplicationError error) async => _handleError(error),
    );
  }

  FutureOr<void> _onUserOptOut(WalletTransferOptOutEvent event, Emitter<WalletTransferTargetState> emit) {
    // Notify core, UI handles navigation by itself directly after opting out.
    unawaited(_skipWalletTransferUseCase.invoke());
  }

  Future<void> _startObservingSessionState(String qrData) async {
    await _sessionStateSubscription?.cancel();
    _sessionStateSubscription = _observeTransferSessionStateUseCase.invoke().listen(
      (status) {
        switch (status) {
          case TransferSessionState.created:
            add(WalletTransferUpdateStateEvent(WalletTransferAwaitingQrScan(qrData)));
          case TransferSessionState.paired:
            add(const WalletTransferUpdateStateEvent(WalletTransferAwaitingConfirmation()));
          case TransferSessionState.confirmed:
            add(const WalletTransferUpdateStateEvent(WalletTransferTransferring(isReceiving: false)));
          case TransferSessionState.uploaded:
            _startReceiving();
          case TransferSessionState.error:
            final error = GenericError('transfer_error', sourceError: status);
            add(WalletTransferUpdateStateEvent(WalletTransferFailed(error)));
          case TransferSessionState.success:
            add(const WalletTransferUpdateStateEvent(WalletTransferSuccess()));
          case TransferSessionState.cancelled:
            add(const WalletTransferUpdateStateEvent(WalletTransferStopped()));
        }
      },
      onError: (ex) => _handleError(
        tryCast<ApplicationError>(ex) ?? GenericError('transfer_status_stream_error', sourceError: ex),
      ),
    );
  }

  FutureOr<void> _onStopRequested(
    WalletTransferStopRequestedEvent event,
    Emitter<WalletTransferTargetState> emit,
  ) async {
    final result = await _cancelWalletTransferUsecase.invoke();
    // We only want to emit a new state if the wallet is not already in a success/error state
    bool maintainState(WalletTransferTargetState state) => state is WalletTransferSuccess || state is ErrorState;
    await result.process(
      onSuccess: (_) {
        _sessionStateSubscription?.cancel();
        if (maintainState(state)) return;
        emit(const WalletTransferStopped());
      },
      onError: (ex) {
        Fimber.e('Failed to cancel wallet transfer', ex: ex);
        _sessionStateSubscription?.cancel();
        if (maintainState(state)) return;
        _handleError(ex);
      },
    );
  }

  FutureOr<void> _onBackPressed(WalletTransferBackPressedEvent event, Emitter<WalletTransferTargetState> emit) async {
    if (!state.canGoBack) return;
    if (state is WalletTransferAwaitingQrScan) {
      unawaited(_sessionStateSubscription?.cancel());
      emit(const WalletTransferIntroduction(didGoBack: true));
    }
  }

  Future<void> _handleError(ApplicationError error) async {
    switch (error) {
      case NetworkError():
        add(WalletTransferUpdateStateEvent(WalletTransferNetworkError(error)));
      case SessionError():
        add(WalletTransferUpdateStateEvent(WalletTransferSessionExpired(error)));
      default:
        add(WalletTransferUpdateStateEvent(WalletTransferGenericError(error)));
    }
  }

  Future<void> _startReceiving() async {
    final isReceiving = tryCast<WalletTransferTransferring>(state)?.isReceiving ?? false;
    assert(!isReceiving, 'Wallet already in transferring state, should never attempt to receive twice!');
    if (isReceiving) return;

    // Stop polling for transfer status
    await _sessionStateSubscription?.cancel();
    // Move to to isReceiving state
    add(const WalletTransferUpdateStateEvent(WalletTransferTransferring(isReceiving: true)));
    // Start receiving and emit result
    final result = await _receiveWalletTransferUseCase.invoke();
    await result.process(
      onSuccess: (_) => add(const WalletTransferUpdateStateEvent(WalletTransferSuccess())),
      onError: _handleError,
    );
  }

  @override
  Future<void> close() async {
    _autoLockProvider.setAutoLock(enabled: true); // Always re-enable lock onClose
    await _sessionStateSubscription?.cancel();
    return super.close();
  }
}
