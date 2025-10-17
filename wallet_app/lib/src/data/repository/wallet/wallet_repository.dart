import 'package:wallet_core/core.dart';

import '../../../domain/model/wallet_status.dart';

abstract class WalletRepository {
  /// Stream that indicates whether the wallet is currently locked,
  /// meaning the pin should be provided to interact with the wallet.
  Stream<bool> get isLockedStream;

  /// Create a wallet, this will register
  /// the wallet with the wallet provider
  Future<void> createWallet(String pin);

  /// Checks if the wallet is created and
  /// registered at the wallet provider
  Future<bool> isRegistered();

  /// Unlock the wallet, also updates the [isLockedStream] when successful
  Future<WalletInstructionResult> unlockWallet(String pin);

  /// Unlock the wallet **without** pin, requires biometric unlock to be enabled.
  /// Also updates the [isLockedStream] when successful.
  ///
  /// WARNING: Currently `wallet_app` is responsible for verifying biometrics, meaning this method should
  /// WARNING: only be called after verifying the user's biometrics. Instead of calling this method directly
  /// WARNING: it is highly recommended to use [UnlockWalletWithBiometricsUseCase] to reach this method.
  Future<void> unlockWalletWithBiometrics();

  /// Lock the wallet, updates [isLockedStream]
  Future<void> lockWallet();

  /// Check if the wallet contains the PID card
  Future<bool> containsPid();

  /// Resets the wallet, i.e. removes cards & registration.
  Future<void> resetWallet();

  /// Fetch the current status of the wallet
  Future<WalletStatus> getWalletStatus();
}
