import 'dart:async';

import 'package:fimber/fimber.dart';
import 'package:rxdart/rxdart.dart';

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
  Stream<TransferSessionState> invoke() {
    TransferSessionState? status; // Keep track of the status, so we can auto-close the stream on [_terminalStates].
    final startStream = Stream.value(null); // Stream that fires instantly, so we instantly emit a state
    final timerStream = Stream.periodic(const Duration(seconds: 2)); // Fetch new state updates every 2 seconds
    final notifyDoneStream = StreamController(); // Allows us to end stream instantly (triggering takeWhile check)
    final intervalStream = MergeStream([timerStream, startStream, notifyDoneStream.stream]);
    return intervalStream.takeWhile((_) => !_terminalStates.contains(status)).asyncMap(
      /* asyncMap does not start processing next item until this future completes */
      (_) async {
        try {
          status = await _transferRepository.getWalletTransferState();
          return status!;
        } catch (ex) {
          Fimber.e('Failed to get transfer state', ex: ex);
          throw (await WalletUseCase.exceptionToApplicationError(ex));
        } finally {
          // Makes sure we don't wait until the next timerStream event, causing the stream to complete immediately.
          if (_terminalStates.contains(status)) notifyDoneStream.add(null);
        }
      },
    );
  }

  Stream<TransferSessionState> observeTransferSessionState() async* {
    while (true) {
      final status = await _transferRepository.getWalletTransferState();
      yield status;
      if (_terminalStates.contains(status)) return;
      await Future.delayed(const Duration(seconds: 2));
    }
  }
}
