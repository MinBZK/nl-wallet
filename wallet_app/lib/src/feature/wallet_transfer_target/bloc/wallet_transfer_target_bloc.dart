import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/bloc/error_state.dart';
import '../../../domain/model/bloc/network_error_state.dart';
import '../../../domain/model/flow_progress.dart';
import '../../../domain/model/result/application_error.dart';
import '../../../domain/model/transfer/wallet_transfer_status.dart';
import '../../../domain/usecase/transfer/cancel_wallet_transfer_usecase.dart';
import '../../../domain/usecase/transfer/get_wallet_transfer_status_usecase.dart';
import '../../../domain/usecase/transfer/init_wallet_transfer_usecase.dart';
import '../../../domain/usecase/transfer/skip_wallet_transfer_usecase.dart';
import '../../../util/cast_util.dart';

part 'wallet_transfer_target_event.dart';
part 'wallet_transfer_target_state.dart';

class WalletTransferTargetBloc extends Bloc<WalletTransferTargetEvent, WalletTransferTargetState> {
  final InitWalletTransferUseCase _initWalletTransferUseCase;
  final GetWalletTransferStatusUseCase _getWalletTransferStatusUseCase;
  final CancelWalletTransferUseCase _cancelWalletTransferUsecase;
  final SkipWalletTransferUseCase _skipWalletTransferUseCase;

  StreamSubscription? _statusSubscription;

  WalletTransferTargetBloc(
    this._initWalletTransferUseCase,
    this._getWalletTransferStatusUseCase,
    this._cancelWalletTransferUsecase,
    this._skipWalletTransferUseCase,
  ) : super(const WalletTransferIntroduction()) {
    on<WalletTransferRestartEvent>(_onUserRestart);
    on<WalletTransferOptInEvent>(_onUserOptIn);
    on<WalletTransferOptOutEvent>(_onUserOptOut);
    on<WalletTransferStopRequestedEvent>(_onStopRequested);
    on<WalletTransferBackPressedEvent>(_onBackPressed);
  }

  FutureOr<void> _onUserRestart(WalletTransferRestartEvent event, Emitter<WalletTransferTargetState> emit) {
    unawaited(_statusSubscription?.cancel());
    emit(const WalletTransferIntroduction(didGoBack: true));
  }

  Future<void> _onUserOptIn(
    WalletTransferOptInEvent event,
    Emitter<WalletTransferTargetState> emit,
  ) async {
    emit(const WalletTransferLoadingQrData());
    final result = await _initWalletTransferUseCase.invoke();
    await result.process(
      onSuccess: (qrData) async {
        if (state is! WalletTransferLoadingQrData) return; // User cancelled
        emit(WalletTransferAwaitingQrScan(qrData));
        await _startObservingStatus(qrData, emit);
      },
      onError: (ApplicationError error) async => _handleError(error, emit),
    );
  }

  FutureOr<void> _onUserOptOut(WalletTransferOptOutEvent event, Emitter<WalletTransferTargetState> emit) {
    // Notify core, UI handles navigation by itself directly after opting out.
    unawaited(_skipWalletTransferUseCase.invoke());
  }

  FutureOr<void> _startObservingStatus(String qrData, Emitter<WalletTransferTargetState> emit) async {
    await _statusSubscription?.cancel();
    _statusSubscription = _getWalletTransferStatusUseCase.invoke().listen((status) {
      switch (status) {
        case WalletTransferStatus.waitingForScan:
          emit(WalletTransferAwaitingQrScan(qrData));
        case WalletTransferStatus.waitingForApproval:
          emit(const WalletTransferAwaitingConfirmation());
        case WalletTransferStatus.transferring:
          emit(const WalletTransferTransferring());
        case WalletTransferStatus.error:
          emit(WalletTransferFailed(GenericError('transfer_error', sourceError: status)));
        case WalletTransferStatus.success:
          emit(const WalletTransferSuccess());
        case WalletTransferStatus.cancelled:
          emit(const WalletTransferStopped());
      }
    });

    try {
      // Await the stream, this way the on<...> listener stays active and is allowed to emit states
      await _statusSubscription?.asFuture(() {});
    } catch (ex) {
      Fimber.e('Status stream failed', ex: ex);
      await _handleError(tryCast<ApplicationError>(ex) ?? GenericError('status_stream', sourceError: ex), emit);
    }
  }

  FutureOr<void> _onStopRequested(
    WalletTransferStopRequestedEvent event,
    Emitter<WalletTransferTargetState> emit,
  ) async {
    unawaited(_statusSubscription?.cancel());
    await _cancelWalletTransferUsecase.invoke();
    emit(const WalletTransferStopped());
  }

  FutureOr<void> _onBackPressed(WalletTransferBackPressedEvent event, Emitter<WalletTransferTargetState> emit) async {
    if (!state.canGoBack) return;
    if (state is WalletTransferAwaitingQrScan) {
      unawaited(_statusSubscription?.cancel());
      emit(const WalletTransferIntroduction(didGoBack: true));
    }
  }

  Future<void> _handleError(ApplicationError error, Emitter<WalletTransferTargetState> emit) async {
    switch (error) {
      case NetworkError():
        emit(WalletTransferNetworkError(error));
      case SessionError():
        emit(WalletTransferSessionExpired(error));
      default:
        emit(WalletTransferGenericError(error));
    }
  }

  @override
  Future<void> close() async {
    await _statusSubscription?.cancel();
    return super.close();
  }
}
