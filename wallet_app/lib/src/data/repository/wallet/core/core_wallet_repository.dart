import 'dart:async';
import 'dart:io' as io;

import 'package:fimber/fimber.dart';
import 'package:flutter/cupertino.dart';
import 'package:sentry_flutter/sentry_flutter.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/wallet_state.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../wallet_repository.dart';

typedef ExitFn = Function(int code);

final walletNotRegisteredError = StateError('Wallet not yet registered!');

class CoreWalletRepository implements WalletRepository {
  @visibleForTesting
  ExitFn exit = io.exit;

  final Mapper<core.WalletState, WalletState> _stateMapper;
  final TypedWalletCore _walletCore;

  CoreWalletRepository(this._walletCore, this._stateMapper);

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
  Future<core.WalletInstructionResult> unlockWallet(String pin) async {
    if (!(await isRegistered())) throw walletNotRegisteredError;
    return _walletCore.unlockWallet(pin);
  }

  @override
  Future<void> unlockWalletWithBiometrics() async {
    if (!(await isRegistered())) throw walletNotRegisteredError;
    return _walletCore.unlockWithBiometrics();
  }

  @override
  Future<bool> containsPid() async {
    try {
      final cards = await _walletCore.observeCards().first.timeout(const Duration(seconds: 1));
      // Since cards can't be issued without having the PID, having any card means we have the PID.
      return cards.isNotEmpty;
    } on TimeoutException catch (ex) {
      Fimber.e('Stream remained empty, no cards available yet.', ex: ex);
      return false;
    }
  }

  @override
  Future<WalletState> getWalletState() async {
    final state = await _walletCore.getWalletState();
    return _stateMapper.map(state);
  }
}
