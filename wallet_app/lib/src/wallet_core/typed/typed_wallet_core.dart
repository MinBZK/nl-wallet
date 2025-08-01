import 'dart:async';
import 'dart:io';

import 'package:fimber/fimber.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:rxdart/rxdart.dart';
import 'package:sentry_flutter/sentry_flutter.dart';
import 'package:wallet_core/core.dart' as core;

import '../../util/mapper/mapper.dart';
import '../error/core_error.dart';

/// Wraps the [WalletCore].
/// Adds auto initialization, pass through of the locked
/// flag and parsing of the [FlutterApiError]s.
class TypedWalletCore {
  final Mapper<String, CoreError> _errorMapper;
  final BehaviorSubject<bool> _isLocked = BehaviorSubject.seeded(true);
  final BehaviorSubject<core.FlutterConfiguration> _flutterConfig = BehaviorSubject();
  final BehaviorSubject<core.FlutterVersionState> _flutterVersionState = BehaviorSubject();
  final BehaviorSubject<List<core.WalletEvent>> _recentHistory = BehaviorSubject();
  final BehaviorSubject<List<core.AttestationPresentation>> _attestations = BehaviorSubject();

  TypedWalletCore(this._errorMapper) {
    _setupLockedStream();
    _setupConfigurationStream();
    _setupVersionStateStream();
    _setupAttestationsStream();
    _setupRecentHistoryStream();
  }

  void _setupLockedStream() {
    _isLocked.onListen = () async {
      core.setLockStream().listen(_isLocked.add);
    };
    _isLocked.onCancel = core.clearLockStream;
  }

  void _setupConfigurationStream() {
    _flutterConfig.onListen = () async {
      core.setConfigurationStream().listen(_flutterConfig.add);
    };
    _flutterConfig.onCancel = core.clearConfigurationStream;
  }

  void _setupVersionStateStream() {
    _flutterVersionState.onListen = () async {
      core.setVersionStateStream().listen(_flutterVersionState.add);
    };
    _flutterVersionState.onCancel = core.clearVersionStateStream;
  }

  Future<void> _setupAttestationsStream() async {
    // Ideally we don't set the card stream until we start observing it (i.e. in onListen())
    // but since the cards are not persisted yet that means we might miss events, so observing
    // the wallet_core cards stream through the complete lifecycle of the app for now.
    // NOTE: To reproduce issue: 1. Start clean, 2. Setup Wallet, 3. Kill app, 4. Continue Setup, 5. Cards don't show up on success page
    core.setAttestationsStream().listen(_attestations.add);
  }

  void _setupRecentHistoryStream() {
    _recentHistory.onListen = () async {
      core.setRecentHistoryStream().listen(_recentHistory.add);
    };
    _recentHistory.onCancel = core.clearRecentHistoryStream;
  }

  Future<core.PinValidationResult> isValidPin(String pin) => call(() => core.isValidPin(pin: pin));

  Future<void> register(String pin) => call(() => core.register(pin: pin));

  Future<bool> isRegistered() => call(core.hasRegistration);

  Future<void> lockWallet() => call(core.lockWallet);

  Future<core.WalletInstructionResult> unlockWallet(String pin) => call(() => core.unlockWallet(pin: pin));

  Future<core.WalletInstructionResult> checkPin(String pin) => call(() => core.checkPin(pin: pin));

  Future<core.WalletInstructionResult> changePin(String oldPin, newPin) =>
      call(() => core.changePin(oldPin: oldPin, newPin: newPin));

  Future<core.WalletInstructionResult> continueChangePin(String pin) => call(() => core.continueChangePin(pin: pin));

  Stream<bool> get isLocked => _isLocked;

  Stream<core.FlutterConfiguration> observeConfig() => _flutterConfig.stream;

  Stream<core.FlutterVersionState> observeVersionState() => _flutterVersionState.stream;

  Future<String> createPidIssuanceRedirectUri() => call(core.createPidIssuanceRedirectUri);

  Future<String> createPidRenewalRedirectUri() => call(core.createPidRenewalRedirectUri);

  Future<core.IdentifyUriResult> identifyUri(String uri) => call(() => core.identifyUri(uri: uri));

  /// Cancel ongoing issuance session (includes PID issuance)
  Future<void> cancelIssuance() => call(core.cancelIssuance);

  Future<List<core.AttestationPresentation>> continuePidIssuance(String uri) =>
      call(() => core.continuePidIssuance(uri: uri));

  Future<core.DisclosureBasedIssuanceResult> continueDisclosureBasedIssuance(String pin) =>
      call(() => core.continueDisclosureBasedIssuance(pin: pin));

  /// Accept offered attestations (includes PID)
  Future<core.WalletInstructionResult> acceptIssuance(String pin) => call(() => core.acceptIssuance(pin: pin));

  /// Check if there is an active issuance session (includes PID issuance)
  Future<bool> hasActiveIssuanceSession() => call(core.hasActiveIssuanceSession);

  Future<core.StartDisclosureResult> startDisclosure(
    String uri, {
    bool isQrCode = false,
  }) =>
      call(() => core.startDisclosure(uri: uri, isQrCode: isQrCode));

  Future<String?> cancelDisclosure() => call(core.cancelDisclosure);

  Future<core.AcceptDisclosureResult> acceptDisclosure(String pin) => call(() => core.acceptDisclosure(pin: pin));

  Future<bool> hasActiveDisclosureSession() => call(core.hasActiveDisclosureSession);

  Stream<List<core.AttestationPresentation>> observeCards() => _attestations.stream;

  Future<void> resetWallet() => call(core.resetWallet);

  Future<List<core.WalletEvent>> getHistory() => call(core.getHistory);

  Future<List<core.WalletEvent>> getHistoryForCard(String attestationId) =>
      call(() => core.getHistoryForCard(attestationId: attestationId));

  Stream<List<core.WalletEvent>> observeRecentHistory() => _recentHistory.stream;

  Future<bool> isBiometricLoginEnabled() => call(core.isBiometricUnlockEnabled);

  Future<void> setBiometricUnlock({required bool enabled}) => call(() => core.setBiometricUnlock(enable: enabled));

  Future<void> unlockWithBiometrics() => call(core.unlockWalletWithBiometrics);

  Future<String> getVersionString() => call(core.getVersionString);

  /// This function should be used to call through to the core, as it makes sure potential exceptions are processed
  /// before they are (re)thrown.
  Future<T> call<T>(Future<T> Function() runnable) async {
    try {
      return await runnable();
    } catch (exception, stacktrace) {
      throw await _handleCoreException(exception, stackTrace: stacktrace);
    }
  }

  /// Converts the exception to a [CoreError] if it can be mapped into one, otherwise returns the original exception.
  Future<Object> _handleCoreException(
    Object ex, {
    StackTrace? stackTrace,
  }) async {
    try {
      final String coreErrorJson = _extractErrorJson(ex)!;
      final error = _errorMapper.map(coreErrorJson);
      if (error is CoreStateError) {
        Fimber.e(
          'StateError detected, this indicates a programming error. Crashing...',
          ex: error,
          stacktrace: stackTrace,
        );
        await Sentry.captureException(error, stackTrace: stackTrace);
        exit(0);
      }
      return error;
    } catch (exception) {
      Fimber.e(
        'Failed to map exception to CoreError, returning original exception',
        ex: exception,
      );
      return ex;
    }
  }

  String? _extractErrorJson(Object ex) {
    if (ex is AnyhowException) {
      Fimber.e('AnyhowException. Contents: ${ex.message}');
      return ex.message;
    } else if (ex is String) {
      Fimber.e('StringException. Contents: $ex');
      return ex;
    }
    Fimber.d('Unable to extract error json from: ${ex.runtimeType}');
    return null;
  }
}
