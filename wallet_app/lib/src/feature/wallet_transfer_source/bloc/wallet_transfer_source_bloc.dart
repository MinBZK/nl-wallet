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
import '../../../domain/usecase/transfer/observe_transfer_session_state_usecase.dart';
import '../../../domain/usecase/transfer/pair_wallet_transfer_usecase.dart';
import '../../../domain/usecase/transfer/start_wallet_transfer_usecase.dart';
import '../../../util/cast_util.dart';

part 'wallet_transfer_source_event.dart';
part 'wallet_transfer_source_state.dart';

class WalletTransferSourceBloc extends Bloc<WalletTransferSourceEvent, WalletTransferSourceState> {
  final PairWalletTransferUseCase _ackWalletTransferUseCase;
  final ObserveTransferSessionStateUseCase _observeTransferSessionStateUseCase;
  final CancelWalletTransferUseCase _cancelWalletTransferUsecase;
  final StartWalletTransferUseCase _startWalletTransferUseCase;
  final AutoLockService _autoLockService;

  StreamSubscription? _sessionStateSubscription;

  WalletTransferSourceBloc(
    this._ackWalletTransferUseCase,
    this._observeTransferSessionStateUseCase,
    this._cancelWalletTransferUsecase,
    this._startWalletTransferUseCase,
    this._autoLockService,
  ) : super(const WalletTransferInitial()) {
    on<WalletTransferAcknowledgeTransferEvent>(_onAcknowledgeTransfer);
    on<WalletTransferAgreeEvent>(_onTermsAccepted);
    on<WalletTransferPinConfirmedEvent>(_onPinConfirmed);
    on<WalletTransferStopRequestedEvent>(_onStopRequested);
    on<WalletTransferBackPressedEvent>(_onBackPressed);
    on<WalletTransferPinConfirmationFailed>(_onPinConfirmationFailed);
    on<WalletTransferUpdateStateEvent>((event, emit) => emit(event.state));
  }

  @override
  void onChange(Change<WalletTransferSourceState> change) {
    super.onChange(change);
    switch (change.nextState) {
      case WalletTransferConfirmPin():
      case WalletTransferTransferring():
      case WalletTransferSuccess():
        _autoLockService.setAutoLock(enabled: false);
      case WalletTransferInitial():
      case WalletTransferLoading():
      case WalletTransferIntroduction():
      case WalletTransferStopped():
      case WalletTransferGenericError():
      case WalletTransferNetworkError():
      case WalletTransferSessionExpired():
      case WalletTransferFailed():
        _autoLockService.setAutoLock(enabled: true);
    }
  }

  Future<void> _onAcknowledgeTransfer(
    WalletTransferAcknowledgeTransferEvent event,
    Emitter<WalletTransferSourceState> emit,
  ) async {
    emit(const WalletTransferLoading());
    final result = await _ackWalletTransferUseCase.invoke(event.uri);
    await result.process(
      onSuccess: (_) {
        emit(const WalletTransferIntroduction());
        _startObservingSessionState();
      },
      onError: _handleError,
    );
  }

  FutureOr<void> _onTermsAccepted(WalletTransferAgreeEvent event, Emitter<WalletTransferSourceState> emit) async {
    emit(const WalletTransferConfirmPin());
  }

  FutureOr<void> _onPinConfirmed(WalletTransferPinConfirmedEvent event, Emitter<WalletTransferSourceState> emit) async {
    emit(const WalletTransferTransferring());
    final result = await _startWalletTransferUseCase.invoke();
    await result.process(
      onSuccess: (_) => Fimber.d('transferWallet (upload) success'),
      onError: (error) async {
        try {
          // PVW-5194 (startWalletTransferUseCase fails when session is stopped on destination)
          final transferSessionState = await _observeTransferSessionStateUseCase.invoke().first;
          if (transferSessionState == TransferSessionState.cancelled) emit(const WalletTransferStopped());
        } catch (e) {
          Fimber.i('Failed to get state. This happens if cancelled was already reached through polling.', ex: e);
        } finally {
          if (state is! WalletTransferStopped) unawaited(_handleError(error));
        }
      },
    );
  }

  FutureOr<void> _onStopRequested(
    WalletTransferStopRequestedEvent event,
    Emitter<WalletTransferSourceState> emit,
  ) async {
    final result = await _cancelWalletTransferUsecase.invoke();
    // We only want to emit a new state if the wallet is not already in a success/error state
    bool maintainState(WalletTransferSourceState state) => state is WalletTransferSuccess || state is ErrorState;
    await result.process(
      onSuccess: (_) {
        _stopObservingSessionState();
        if (maintainState(state)) return;
        emit(const WalletTransferStopped());
      },
      onError: (ex) {
        Fimber.e('Failed to cancel wallet transfer', ex: ex);
        _stopObservingSessionState();
        if (maintainState(state)) return;
        _handleError(ex);
      },
    );
  }

  FutureOr<void> _onBackPressed(WalletTransferBackPressedEvent event, Emitter<WalletTransferSourceState> emit) async {
    if (!state.canGoBack) return;
    if (state is WalletTransferConfirmPin) emit(const WalletTransferIntroduction(didGoBack: true));
  }

  FutureOr<void> _onPinConfirmationFailed(
    WalletTransferPinConfirmationFailed event,
    Emitter<WalletTransferSourceState> emit,
  ) => _handleError(event.error);

  Future<void> _handleError(ApplicationError error) async {
    _stopObservingSessionState();
    switch (error) {
      case NetworkError():
        add(WalletTransferUpdateStateEvent(WalletTransferNetworkError(error)));
      case SessionError():
        add(WalletTransferUpdateStateEvent(WalletTransferSessionExpired(error)));
      default:
        add(WalletTransferUpdateStateEvent(WalletTransferGenericError(error)));
    }
  }

  Future<void> _startObservingSessionState() async {
    await _sessionStateSubscription?.cancel();
    _sessionStateSubscription = _observeTransferSessionStateUseCase.invoke().listen(
      (status) {
        switch (status) {
          case TransferSessionState.created:
          case TransferSessionState.paired:
          case TransferSessionState.confirmed:
            break;
          case TransferSessionState.uploaded:
            add(const WalletTransferUpdateStateEvent(WalletTransferTransferring()));
          case TransferSessionState.error:
            final walletTransferFailed = WalletTransferFailed(GenericError('transfer_error', sourceError: status));
            add(WalletTransferUpdateStateEvent(walletTransferFailed));
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

  void _stopObservingSessionState() {
    _sessionStateSubscription?.cancel();
    _sessionStateSubscription = null;
  }

  @override
  Future<void> close() async {
    _autoLockService.setAutoLock(enabled: true);
    _stopObservingSessionState();
    return super.close();
  }
}
