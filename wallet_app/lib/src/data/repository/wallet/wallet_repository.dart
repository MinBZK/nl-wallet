import '../../../../bridge_generated.dart';
import '../../../domain/usecase/pin/unlock_wallet_with_pin_usecase.dart';

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

  /// Lock the wallet, updates [isLockedStream]
  void lockWallet();

  /// Confirm a transaction
  Future<CheckPinResult> confirmTransaction(String pin);

  /// Check if the wallet contains the PID card
  Future<bool> containsPid();

  /// Resets the wallet, i.e. removes cards & registration.
  Future<void> resetWallet();
}
