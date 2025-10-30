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

  // ignore: unused_field - Will be used once event log supports transfer events.
  final WalletEventLog _eventLog;

  TransferSessionState _currentState = TransferSessionState.Canceled;

  TransferManager(this._pinManager, this._wallet, this._eventLog);

  Future<WalletInstructionResult> confirmWalletTransfer(String pin) async {
    final result = _pinManager.checkPin(pin);
    final bool pinMatches = result is WalletInstructionResult_Ok;
    if (pinMatches) _currentState = TransferSessionState.Confirmed;
    return result;
  }

  Future<void> transferWallet() async {
    await Future.delayed(const Duration(seconds: 3));
    _currentState = TransferSessionState.Success;
  }

  void pairWalletTransfer(String uri) {
    isSourceDevice = true;
    _currentState = TransferSessionState.Paired;
  }

  Future<String> initWalletTransfer() async {
    isSourceDevice = false;
    _currentState = TransferSessionState.Created;
    return 'QR_CODE_CONTENTS';
  }

  void cancelWalletTransfer() => _currentState = TransferSessionState.Canceled;

  Future<TransferSessionState> getTransferState() async {
    final currentState = _currentState;

    // Mock state transitions for the next time this is polled
    switch (currentState) {
      case TransferSessionState.Created:
        _currentState = TransferSessionState.Paired;
      case TransferSessionState.Paired:
        _currentState = TransferSessionState.Uploaded;
      case TransferSessionState.Uploaded:
        _currentState = TransferSessionState.Success;
      case TransferSessionState.Confirmed:
        break; // Awaiting call to confirmWalletTransfer()
      case TransferSessionState.Success:
        // Log successful transfer event
        if (isSourceDevice) {
          _wallet.reset();
          _wallet.unlock(); // Avoid showing unlock overlay
        }
      case TransferSessionState.Canceled:
      case TransferSessionState.Error:
        log('Terminal state, no transition needed');
    }
    return currentState;
  }
}
