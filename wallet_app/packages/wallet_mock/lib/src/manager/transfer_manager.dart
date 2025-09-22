import 'dart:developer';

import 'package:wallet_core/core.dart';

import '../log/wallet_event_log.dart';
import '../wallet/wallet.dart';
import 'pin_manager.dart';

class TransferManager {
  // Deduced by ack/init calls
  bool isSourceDevice = false;

  final PinManager _pinManager;
  final Wallet _wallet;
  final WalletEventLog _eventLog;

  TransferSessionState _currentState = TransferSessionState.Cancelled;

  TransferManager(this._pinManager, this._wallet, this._eventLog);

  Future<WalletInstructionResult> transferWallet(String pin) async {
    final result = _pinManager.checkPin(pin);
    final bool pinMatches = result is WalletInstructionResult_Ok;
    if (pinMatches) _currentState = TransferSessionState.ReadyForTransfer;
    return result;
  }

  void acknowledgeWalletTransfer(String uri) {
    isSourceDevice = true;
    _currentState = TransferSessionState.Created;
  }

  Future<String> initWalletTransfer() async {
    isSourceDevice = false;
    _currentState = TransferSessionState.Created;
    return 'QR_CODE_CONTENTS';
  }

  void cancelWalletTransfer() => _currentState = TransferSessionState.Cancelled;

  Future<TransferSessionState> getTransferState() async {
    final currentState = _currentState;

    // Mock state transitions for the next time this is polled
    switch (currentState) {
      case TransferSessionState.Created:
        _currentState = TransferSessionState.ReadyForTransfer;
      case TransferSessionState.ReadyForTransfer:
        _currentState = TransferSessionState.ReadyForDownload;
      case TransferSessionState.ReadyForDownload:
        _currentState = TransferSessionState.Success;
      case TransferSessionState.Success:
        // Log successful transfer event
        if (isSourceDevice) {
          _wallet.reset();
          _wallet.unlock(); // Avoid showing unlock overlay
        }
      case TransferSessionState.Cancelled:
      case TransferSessionState.Error:
        log('Terminal state, no transition needed');
    }
    return currentState;
  }
}
