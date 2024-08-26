import 'package:wallet_core/core.dart';

abstract class WalletRepository {
  /// Stream that indicates whether the wallet is currently locked,
  /// meaning the pin should be provided to interact with the wallet.
  Stream<bool> get isLockedStream;

  /// Validates the supplied [pin]
  ///
  /// Throws a [PinValidationError] if the pin does
  /// not meet the required standards.
  Future<void> validatePin(String pin);

  /// Create a wallet, this will register
  /// the wallet with the wallet provider
  Future<void> createWallet(String pin);

  /// Checks if the wallet is created and
  /// registered at the wallet provider
  Future<bool> isRegistered();

  /// Unlock the wallet, also updates the [isLockedStream] when successful
  Future<WalletInstructionResult> unlockWallet(String pin);

  /// Check if the provided pin matches the one that is registered
  Future<WalletInstructionResult> checkPin(String pin);

  // Changes the registered pin to the provided [newPin] it the currently registered pin matches [oldPin]
  Future<WalletInstructionResult> changePin(String oldPin, String newPin);

  /// Lock the wallet, updates [isLockedStream]
  Future<void> lockWallet();

  /// Check if the wallet contains the PID card
  Future<bool> containsPid();

  /// Resets the wallet, i.e. removes cards & registration.
  Future<void> resetWallet();
}
