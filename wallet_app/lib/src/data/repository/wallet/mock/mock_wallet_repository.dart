import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:rxdart/rxdart.dart';

import '../../../../../bridge_generated.dart';
import '../../../../domain/model/pin/pin_validation_error.dart';
import '../../../../domain/usecase/pin/confirm_transaction_usecase.dart';
import '../../../../util/extension/wallet_instruction_result_extension.dart';
import '../../../../wallet_constants.dart';
import '../../../../wallet_core/error/flutter_api_error.dart';
import '../../../source/wallet_datasource.dart';
import '../wallet_repository.dart';

const _kTimeoutUnlockAttempts = 4;
@visibleForTesting
const kMaxUnlockAttempts = 6;

class MockWalletRepository implements WalletRepository {
  final WalletDataSource _dataSource;

  String? _pin;

  /// The amount of times the user incorrectly entered the pin, resets to 0 on a successful attempt.
  int _invalidPinAttempts = 0;
  final BehaviorSubject<bool> _locked = BehaviorSubject<bool>.seeded(true);
  final BehaviorSubject<bool> _isInitialized = BehaviorSubject<bool>.seeded(false);

  MockWalletRepository(this._dataSource);

  @override
  void lockWallet() => _locked.add(true);

  @override
  Future<WalletInstructionResult> unlockWallet(String pin) async {
    if (!isInitialized) throw UnsupportedError('Wallet not yet initialized!');
    await Future.delayed(const Duration(milliseconds: 500));
    if (kDebugMode && pin != _pin) {
      /// This allows us to test all expected errors (and UI states) on mock builds.
      switch (pin) {
        case '111111':
          return const WalletInstructionResult.timeout(timeoutMillis: 10000);
        case '222222':
          return const WalletInstructionResult.incorrectPin(leftoverAttempts: 5, isFinalAttempt: false);
        case '333333':
          return const WalletInstructionResult.blocked();
        case '444444':
          throw FlutterApiError(type: FlutterApiErrorType.generic);
        case '555555':
          throw FlutterApiError(type: FlutterApiErrorType.networking);
      }
    }
    if (_pin != null && pin == _pin) {
      _locked.add(false);
      _invalidPinAttempts = 0;
      return const WalletInstructionResult.ok();
    } else {
      return _handlePinInvalid();
    }
  }

  @override
  Future<CheckPinResult> confirmTransaction(String pin) async {
    if (!isInitialized) throw UnsupportedError('Wallet not yet initialized!');
    if (isLocked) throw UnsupportedError('Wallet is locked');
    if (_pin != null && pin == _pin) {
      _invalidPinAttempts = 0;
      return CheckPinResultOk();
    } else {
      return _handlePinInvalid().asCheckPinResult();
    }
  }

  /// Increase the invalid pin counter and resolve
  /// the currently relevant [WalletInstructionResult].
  WalletInstructionResult _handlePinInvalid() {
    _invalidPinAttempts++;
    // Ugly & long, but also temporary
    if (_invalidPinAttempts <= _kTimeoutUnlockAttempts) {
      if (_invalidPinAttempts >= _kTimeoutUnlockAttempts) {
        // Trigger timeout
        return const WalletInstructionResult.timeout(timeoutMillis: 15 * 1000 /* 15 seconds */);
      } else {
        // Trigger normal (pre timeout) attempts left
        return WalletInstructionResult.incorrectPin(
          leftoverAttempts: _kTimeoutUnlockAttempts - _invalidPinAttempts,
          isFinalAttempt: false,
        );
      }
    } else {
      // After initial timeout (user only gets 1 timeout in mock)
      if (_invalidPinAttempts >= kMaxUnlockAttempts) {
        // Too many attempts, block user
        resetWallet();
        return const WalletInstructionResult.blocked();
      } else {
        // x Attempts left in final round
        var attemptsLeft = kMaxUnlockAttempts - _invalidPinAttempts;
        return WalletInstructionResult.incorrectPin(
          leftoverAttempts: attemptsLeft,
          isFinalAttempt: attemptsLeft == 1,
        );
      }
    }
  }

  @override
  Future<void> createWallet(String pin) async {
    if (pin.length != kPinDigits) throw UnsupportedError('Invalid pin. Length should be $kPinDigits');
    if (isInitialized) throw UnsupportedError('Wallet is already initialized');
    await Future.delayed(kDefaultMockDelay);
    _pin = pin;
    _isInitialized.add(true);
    _invalidPinAttempts = 0;
    _locked.add(false);
  }

  Stream<bool> get isInitializedStream => _isInitialized.stream.distinct();

  bool get isInitialized => _isInitialized.value;

  @override
  Future<bool> isRegistered() async => isInitialized;

  bool get isLocked => _locked.value;

  @override
  Stream<bool> get isLockedStream => _locked.stream.distinct();

  @override
  Future<void> validatePin(String pin) async {
    if (pin.length != kPinDigits) throw PinValidationError.other;
    if (pin.characters.toSet().length <= 1) throw PinValidationError.tooFewUniqueDigits;
    if (pin == '123456') throw PinValidationError.sequentialDigits;
  }

  @override
  Future<bool> containsPid() async => (await _dataSource.read('PID_1')) != null;

  @override
  Future<void> resetWallet() async {
    if (!isInitialized) throw UnsupportedError('Wallet not yet initialized!');
    _dataSource.clear();
    _pin == null;
    _isInitialized.add(false);
    _invalidPinAttempts = 0;
    _locked.add(true);
  }
}
