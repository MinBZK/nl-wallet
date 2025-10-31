import '../../../../data/repository/transfer/transfer_repository.dart';
import '../../../model/transfer/transfer_session_state.dart';
import '../../wallet_usecase.dart';
import '../observe_transfer_session_state_usecase.dart';

/// Use case for observing the [TransferSessionState].
///
/// This class polls the [TransferRepository] for the transfer session state
/// and yields the state until a terminal state is reached.
class ObserveTransferSessionStateUseCaseImpl extends ObserveTransferSessionStateUseCase {
  final TransferRepository _transferRepository;

  static const List<TransferSessionState> _terminalStates = [
    TransferSessionState.success,
    TransferSessionState.cancelled,
    TransferSessionState.error,
  ];

  ObserveTransferSessionStateUseCaseImpl(this._transferRepository);

  @override
  Stream<TransferSessionState> invoke() => observeWalletStatus().handleAppError('Failed to get transfer session state');

  Stream<TransferSessionState> observeWalletStatus() async* {
    while (true) {
      final status = await _transferRepository.getWalletTransferState();
      yield status;
      if (_terminalStates.contains(status)) return;
      await Future.delayed(const Duration(seconds: 2));
    }
  }
}
