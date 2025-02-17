import 'dart:async';
import 'dart:io' as io;

import 'package:fimber/fimber.dart';
import 'package:flutter/cupertino.dart';
import 'package:sentry_flutter/sentry_flutter.dart';
import 'package:wallet_core/core.dart';

import '../../../../domain/model/pin/pin_validation_error.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../wallet_repository.dart';

typedef ExitFn = Function(int code);

final walletNotRegisteredError = StateError('Wallet not yet registered!');

class CoreWalletRepository implements WalletRepository {
  @visibleForTesting
  ExitFn exit = io.exit;

  final TypedWalletCore _walletCore;
  final Mapper<PinValidationResult, PinValidationError?> _pinValidationErrorMapper;

  CoreWalletRepository(this._walletCore, this._pinValidationErrorMapper);

  @override
  Future<void> validatePin(String pin) async {
    final result = await _walletCore.isValidPin(pin);
    final error = _pinValidationErrorMapper.map(result);
    if (error != null) {
      throw error;
    }
  }

  @override
  Future<void> createWallet(String pin) async {
    await _walletCore.register(pin);
  }

  @override
  Future<void> resetWallet() => _walletCore.resetWallet();

  @override
  Future<bool> isRegistered() => _walletCore.isRegistered();

  @override
  Stream<bool> get isLockedStream => _walletCore.isLocked;

  @override
  Future<void> lockWallet() async {
    try {
      await _walletCore.lockWallet();
    } catch (exception, stackTrace) {
      Fimber.e('Failed to lock wallet', ex: exception);
      await Sentry.captureException(exception, stackTrace: stackTrace);
      exit(1); // Crash if locking fails
    }
  }

  @override
  Future<WalletInstructionResult> unlockWallet(String pin) async {
    if (!(await isRegistered())) throw walletNotRegisteredError;
    return _walletCore.unlockWallet(pin);
  }

  @override
  Future<void> unlockWalletWithBiometrics() async {
    if (!(await isRegistered())) throw walletNotRegisteredError;
    return _walletCore.unlockWithBiometrics();
  }

  @override
  Future<WalletInstructionResult> checkPin(String pin) async {
    if (!(await isRegistered())) throw walletNotRegisteredError;
    return _walletCore.checkPin(pin);
  }

  @override
  Future<WalletInstructionResult> changePin(String oldPin, String newPin) async {
    if (!(await isRegistered())) throw walletNotRegisteredError;
    return _walletCore.changePin(oldPin, newPin);
  }

  @override
  Future<WalletInstructionResult> continueChangePin(String pin) async {
    if (!(await isRegistered())) throw walletNotRegisteredError;
    return _walletCore.continueChangePin(pin);
  }

  @override
  Future<bool> containsPid() async {
    try {
      final attestations = await _walletCore.observeCards().first.timeout(const Duration(seconds: 1));
      return attestations.any((attestation) => attestation.attestationType == kPidDocType);
    } on TimeoutException catch (ex) {
      Fimber.e('Stream remained empty, no cards available yet.', ex: ex);
      return false;
    }
  }
}
