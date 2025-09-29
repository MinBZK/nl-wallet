import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/bloc/error_state.dart';
import '../../../domain/model/bloc/network_error_state.dart';
import '../../../domain/model/flow_progress.dart';
import '../../../domain/model/result/application_error.dart';
import '../../../domain/model/transfer/wallet_transfer_status.dart';
import '../../../domain/usecase/transfer/acknowledge_wallet_transfer_usecase.dart';
import '../../../domain/usecase/transfer/cancel_wallet_transfer_usecase.dart';
import '../../../domain/usecase/transfer/get_wallet_transfer_status_usecase.dart';
import '../../../util/cast_util.dart';

part 'wallet_transfer_source_event.dart';
part 'wallet_transfer_source_state.dart';

class WalletTransferSourceBloc extends Bloc<WalletTransferSourceEvent, WalletTransferSourceState> {
  final AcknowledgeWalletTransferUseCase _ackWalletTransferUseCase;
  final GetWalletTransferStatusUseCase _getWalletTransferStatusUseCase;
  final CancelWalletTransferUseCase _cancelWalletTransferUsecase;

  StreamSubscription? _statusSubscription;

  WalletTransferSourceBloc(
    this._ackWalletTransferUseCase,
    this._getWalletTransferStatusUseCase,
    this._cancelWalletTransferUsecase,
  ) : super(const WalletTransferInitial()) {
    on<WalletTransferAcknowledgeTransferEvent>(_onAcknowledgeTransfer);
    on<WalletTransferAgreeEvent>(_onTermsAccepted);
    on<WalletTransferPinConfirmedEvent>(_onPinConfirmed);
    on<WalletTransferStopRequestedEvent>(_onStopRequested);
    on<WalletTransferBackPressedEvent>(_onBackPressed);
  }

  Future<void> _onAcknowledgeTransfer(
    WalletTransferAcknowledgeTransferEvent event,
    Emitter<WalletTransferSourceState> emit,
  ) async {
    emit(const WalletTransferLoading());
    final result = await _ackWalletTransferUseCase.invoke(event.uri);
    await result.process(
      onSuccess: (_) => emit(const WalletTransferIntroduction()),
      onError: (ApplicationError error) => _handleError(error, emit),
    );
  }

  FutureOr<void> _onTermsAccepted(WalletTransferAgreeEvent event, Emitter<WalletTransferSourceState> emit) async {
    emit(const WalletTransferConfirmPin());
  }

  FutureOr<void> _onPinConfirmed(WalletTransferPinConfirmedEvent event, Emitter<WalletTransferSourceState> emit) async {
    emit(const WalletTransferTransferring());
    await _statusSubscription?.cancel();
    _statusSubscription = _getWalletTransferStatusUseCase.invoke().listen((status) {
      switch (status) {
        case WalletTransferStatus.waitingForScan:
        case WalletTransferStatus.waitingForApprovalAndUpload:
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
    Emitter<WalletTransferSourceState> emit,
  ) async {
    unawaited(_statusSubscription?.cancel());
    await _cancelWalletTransferUsecase.invoke();
    emit(const WalletTransferStopped());
  }

  FutureOr<void> _onBackPressed(WalletTransferBackPressedEvent event, Emitter<WalletTransferSourceState> emit) async {
    if (!state.canGoBack) return;
    if (state is WalletTransferConfirmPin) emit(const WalletTransferIntroduction(didGoBack: true));
  }

  Future<void> _handleError(ApplicationError error, Emitter<WalletTransferSourceState> emit) async {
    switch (error) {
      case NetworkError():
        emit(WalletTransferGenericError(error));
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
