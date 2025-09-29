import 'package:wallet_core/core.dart';

int _kMaxAttempts = 12;
int _kAttemptsBeforeTimeout = 4;

class PinManager {
  String? _selectedPin;
  int _attempts = 0;

  bool get isRegistered => _selectedPin != null;

  void setPin(String pin) {
    if (isRegistered) throw StateError('Pin already configured');
    _selectedPin = pin;
  }

  void updatePin(String pin) {
    if (!isRegistered) throw StateError('No pin registered');
    _selectedPin = pin;
  }

  WalletInstructionResult checkPin(String pin) {
    if (!isRegistered) throw StateError('Cannot unlock before registration');

    // We've already reached our max attempts, notify blocked.
    if (_attempts >= _kMaxAttempts) {
      return const WalletInstructionResult.instructionError(error: WalletInstructionError.blocked());
    }

    // Pin matches, grant access and reset state
    if (pin == _selectedPin) {
      _attempts = 0;
      return const WalletInstructionResult.ok();
    }

    // Increase the nr of attempts and figure out the new state
    _attempts++;
    // Max attempts reached, block the app
    if (_attempts >= _kMaxAttempts) {
      return const WalletInstructionResult.instructionError(error: WalletInstructionError.blocked());
    }
    // Intermediate timeout, report as such
    if (_attempts % _kAttemptsBeforeTimeout == 0) {
      final int timeoutMillis = Duration(seconds: _attempts * 2).inMilliseconds;
      return WalletInstructionResult.instructionError(
        error: WalletInstructionError.timeout(timeoutMillis: BigInt.from(timeoutMillis)),
      );
    }
    // No timeout, not yet blocked, notify about the attempts left
    final attemptsLeftInRound = _kAttemptsBeforeTimeout - (_attempts % _kAttemptsBeforeTimeout);
    final attemptsLeftInTotal = _kMaxAttempts - _attempts;
    final isFinalRound = attemptsLeftInTotal < _kAttemptsBeforeTimeout;
    return WalletInstructionResult.instructionError(
      error: WalletInstructionError.incorrectPin(
        attemptsLeftInRound: attemptsLeftInRound,
        isFinalRound: isFinalRound,
      ),
    );
  }

  Future<void> resetPin() async {
    _attempts = 0;
    _selectedPin = null;
  }
}
