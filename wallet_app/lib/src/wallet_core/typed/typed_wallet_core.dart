import 'dart:async';

import 'package:fimber/fimber.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:rxdart/rxdart.dart';
import 'package:sentry_flutter/sentry_flutter.dart';
import 'package:wallet_core/core.dart' as core;

import '../../util/mapper/mapper.dart';
import '../../util/sentry/sentry_breadcrumbs.dart';
import '../error/core_error.dart';

/// A callback function used to handle errors emitted by the [TypedWalletCore].
///
/// This listener receives a [CoreError] whenever an underlying operation in the
/// Rust core fails and the error is successfully mapped.
typedef CoreErrorListener = void Function(CoreError error);

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
  final BehaviorSubject<List<core.AppNotification>> _notifications = BehaviorSubject();

  /// An optional listener that is notified whenever a [CoreError] occurs during
  /// a wallet operation. This is not an interceptor, [CoreError]s are still thrown.
  CoreErrorListener? _errorListener;

  TypedWalletCore(this._errorMapper) {
    _setupLockedStream();
    _setupConfigurationStream();
    _setupVersionStateStream();
    _setupAttestationsStream();
    _setupRecentHistoryStream();
    _setupNotificationStream();
  }

  void setErrorListener(CoreErrorListener? listener) {
    if (_errorListener != null) Fimber.w('ErrorListener was already set, replacing existing listener...');
    _errorListener = listener;
  }

  void _setupLockedStream() {
    _isLocked.onListen = () => core.setLockStream().listen(_isLocked.add);
    _isLocked.onCancel = core.clearLockStream;
  }

  void _setupConfigurationStream() {
    _flutterConfig.onListen = () => core.setConfigurationStream().listen(_flutterConfig.add);
    _flutterConfig.onCancel = core.clearConfigurationStream;
  }

  void _setupVersionStateStream() {
    _flutterVersionState.onListen = () => core.setVersionStateStream().listen(_flutterVersionState.add);
    _flutterVersionState.onCancel = core.clearVersionStateStream;
  }

  void _setupNotificationStream() {
    _notifications.onListen = () => core.setScheduledNotificationsStream().listen(_notifications.add);
    _notifications.onCancel = core.clearScheduledNotificationsStream;
  }

  void setupNotificationCallback(FutureOr<void> Function(List<(int, core.NotificationType)>) callback) {
    core.clearDirectNotificationsCallback();
    core.setDirectNotificationsCallback(callback: callback);
  }

  void _setupAttestationsStream() {
    _attestations.onListen = () => core.setAttestationsStream().listen(_attestations.add);
    _attestations.onCancel = core.clearAttestationsStream;
  }

  void _setupRecentHistoryStream() {
    _recentHistory.onListen = () => core.setRecentHistoryStream().listen(_recentHistory.add);
    _recentHistory.onCancel = core.clearRecentHistoryStream;
  }

  Future<core.PinValidationResult> isValidPin(String pin) => call(() => core.isValidPin(pin: pin));

  Future<void> register(String pin) => _callWithFlowBreadcrumb(
    'wallet.register',
    failureCode: 'wallet.fail.register',
    runnable: () => core.register(pin: pin),
  );

  Future<bool> isRegistered() => call(core.hasRegistration);

  Future<void> lockWallet() => _callWithFlowBreadcrumb(
    'wallet.lock',
    failureCode: 'wallet.fail.lock',
    runnable: core.lockWallet,
  );

  Future<core.WalletInstructionResult> unlockWallet(String pin) => _callWithFlowBreadcrumb(
    'wallet.unlock',
    failureCode: 'wallet.fail.unlock',
    runnable: () => core.unlockWallet(pin: pin),
  );

  Future<core.WalletInstructionResult> checkPin(String pin) => call(() => core.checkPin(pin: pin));

  Future<core.WalletInstructionResult> changePin(String oldPin, newPin) => _callWithFlowBreadcrumb(
    'pin.change',
    failureCode: 'pin.fail.change',
    runnable: () => core.changePin(oldPin: oldPin, newPin: newPin),
  );

  Future<core.WalletInstructionResult> continueChangePin(String pin) => _callWithFlowBreadcrumb(
    'pin.change.continue',
    failureCode: 'pin.change.fail.continue',
    runnable: () => core.continueChangePin(pin: pin),
  );

  Stream<bool> get isLocked => _isLocked;

  Stream<core.FlutterConfiguration> observeConfig() => _flutterConfig.stream;

  Stream<core.FlutterVersionState> observeVersionState() => _flutterVersionState.stream;

  Stream<List<core.AppNotification>> observeNotifications() => _notifications.stream;

  Future<String> createPidIssuanceRedirectUri() => _callWithFlowBreadcrumb(
    'pid_issuance.start',
    failureCode: 'pid_issuance.fail.start',
    runnable: core.createPidIssuanceRedirectUri,
  );

  Future<String> createPidRenewalRedirectUri() => _callWithFlowBreadcrumb(
    'pid_renewal.start',
    failureCode: 'pid_renewal.fail.start',
    runnable: core.createPidRenewalRedirectUri,
  );

  Future<core.IdentifyUriResult> identifyUri(String uri) => _callWithFlowBreadcrumb(
    'uri.identify',
    failureCode: 'uri.fail.identify',
    runnable: () => core.identifyUri(uri: uri),
  );

  Future<List<core.AttestationPresentation>> continueIssuance(String uri) => _callWithFlowBreadcrumb(
    'issuance.continue',
    failureCode: 'issuance.fail.continue',
    runnable: () => core.continueIssuance(uri: uri),
  );

  Future<core.DisclosureBasedIssuanceResult> continueDisclosureBasedIssuance(String pin, List<int> selectedIndices) =>
      _callWithFlowBreadcrumb(
        'issuance.disclosure_based.continue',
        failureCode: 'issuance.disclosure_based.fail.continue',
        runnable: () => core.continueDisclosureBasedIssuance(selectedIndices: selectedIndices, pin: pin),
      );

  Future<core.IssuanceStartResult> startIssuanceFromOffer(String offerUri) => _callWithFlowBreadcrumb(
    'issuance.start',
    failureCode: 'issuance.fail.start',
    runnable: () => core.startIssuanceFromOffer(offerUri: offerUri),
  );

  /// Accept offered attestations
  Future<core.WalletInstructionResult> acceptIssuance(String pin) => _callWithFlowBreadcrumb(
    'issuance.accept',
    failureCode: 'issuance.fail.accept',
    runnable: () => core.acceptIssuance(pin: pin),
  );

  /// Accept offered PID
  Future<core.PidIssuanceResult> acceptPidIssuance(String pin) => _callWithFlowBreadcrumb(
    'pid_issuance.accept',
    failureCode: 'pid_issuance.fail.accept',
    runnable: () => core.acceptPidIssuance(pin: pin),
  );

  Future<core.StartDisclosureResult> startDisclosure(
    String uri, {
    bool isQrCode = false,
  }) => _callWithFlowBreadcrumb(
    'disclosure.start',
    failureCode: 'disclosure.fail.start',
    runnable: () => core.startDisclosure(uri: uri, isQrCode: isQrCode),
  );

  Future<String> startCloseProximityDisclosure({
    required FutureOr<void> Function(core.CloseProximityDisclosureFlutterUpdate) callback,
  }) => _callWithFlowBreadcrumb(
    'disclosure.close_proximity.start',
    failureCode: 'disclosure.close_proximity.fail.start',
    runnable: () => core.startCloseProximityDisclosure(callback: callback),
  );

  Future<core.StartDisclosureResult> continueCloseProximityDisclosure() => _callWithFlowBreadcrumb(
    'disclosure.close_proximity.continue',
    failureCode: 'disclosure.close_proximity.fail.continue',
    runnable: core.continueCloseProximityDisclosure,
  );

  Future<String?> cancelSession() => _callWithFlowBreadcrumb(
    'session.cancel',
    failureCode: 'session.fail.cancel',
    runnable: core.cancelSession,
  );

  Future<core.AcceptDisclosureResult> acceptDisclosure(String pin, List<int> selectedIndices) =>
      _callWithFlowBreadcrumb(
        'disclosure.accept',
        failureCode: 'disclosure.fail.accept',
        runnable: () => core.acceptDisclosure(selectedIndices: selectedIndices, pin: pin),
      );

  Stream<List<core.AttestationPresentation>> observeCards() => _attestations.stream;

  Future<core.WalletInstructionResult> deleteAttestation(String pin, String attestationId) => _callWithFlowBreadcrumb(
    'attestation.delete',
    failureCode: 'attestation.fail.delete',
    runnable: () => core.deleteAttestation(pin: pin, attestationId: attestationId),
  );

  Future<void> resetWallet() => _callWithFlowBreadcrumb(
    'wallet.reset',
    failureCode: 'wallet.fail.reset',
    runnable: core.resetWallet,
  );

  Future<List<core.WalletEvent>> getHistory({required int page, required int pageSize}) =>
      call(() => core.getHistory(page: page, pageSize: pageSize));

  Future<List<core.WalletEvent>> getHistoryForCard(String attestationId) =>
      call(() => core.getHistoryForCard(attestationId: attestationId));

  Stream<List<core.WalletEvent>> observeRecentHistory() => _recentHistory.stream;

  Future<bool> isBiometricLoginEnabled() => call(core.isBiometricUnlockEnabled);

  Future<void> setBiometricUnlock({required bool enabled}) => _callWithFlowBreadcrumb(
    'biometrics.set',
    failureCode: 'biometrics.fail.set',
    runnable: () => core.setBiometricUnlock(enable: enabled),
  );

  Future<void> unlockWithBiometrics() => _callWithFlowBreadcrumb(
    'biometrics.unlock',
    failureCode: 'biometrics.fail.unlock',
    runnable: core.unlockWalletWithBiometrics,
  );

  Future<String> getVersionString() => call(core.getVersionString);

  Future<String> createPinRecoveryRedirectUri() => _callWithFlowBreadcrumb(
    'pin_recovery.start',
    failureCode: 'pin_recovery.fail.start',
    runnable: core.createPinRecoveryRedirectUri,
  );

  Future<void> continuePinRecovery(String uri) => _callWithFlowBreadcrumb(
    'pin_recovery.continue',
    failureCode: 'pin_recovery.fail.continue',
    runnable: () => core.continuePinRecovery(uri: uri),
  );

  Future<void> completePinRecovery(String pin) => _callWithFlowBreadcrumb(
    'pin_recovery.complete',
    failureCode: 'pin_recovery.fail.complete',
    runnable: () => core.completePinRecovery(pin: pin),
  );

  Future<void> cancelPinRecovery() => _callWithFlowBreadcrumb(
    'pin_recovery.cancel',
    failureCode: 'pin_recovery.fail.cancel',
    runnable: core.cancelPinRecovery,
  );

  Future<String> initWalletTransfer() => _callWithFlowBreadcrumb(
    'wallet_transfer.start',
    failureCode: 'wallet_transfer.fail.start',
    runnable: core.initWalletTransfer,
  );

  Future<void> pairWalletTransfer(String uri) => _callWithFlowBreadcrumb(
    'wallet_transfer.pair',
    failureCode: 'wallet_transfer.fail.pair',
    runnable: () => core.pairWalletTransfer(uri: uri),
  );

  Future<core.WalletInstructionResult> confirmWalletTransfer(String pin) => _callWithFlowBreadcrumb(
    'wallet_transfer.confirm',
    failureCode: 'wallet_transfer.fail.confirm',
    runnable: () => core.confirmWalletTransfer(pin: pin),
  );

  Future<void> transferWallet() => _callWithFlowBreadcrumb(
    'wallet_transfer.transfer',
    failureCode: 'wallet_transfer.fail.transfer',
    runnable: core.transferWallet,
  );

  Future<void> receiveWalletTransfer() => _callWithFlowBreadcrumb(
    'wallet_transfer.receive',
    failureCode: 'wallet_transfer.fail.receive',
    runnable: core.receiveWalletTransfer,
  );

  Future<void> cancelWalletTransfer() => _callWithFlowBreadcrumb(
    'wallet_transfer.cancel',
    failureCode: 'wallet_transfer.fail.cancel',
    runnable: core.cancelWalletTransfer,
  );

  Future<core.TransferSessionState> getWalletTransferState() => call(core.getWalletTransferState);

  Future<void> skipWalletTransfer() => _callWithFlowBreadcrumb(
    'wallet_transfer.skip',
    failureCode: 'wallet_transfer.fail.skip',
    runnable: core.skipWalletTransfer,
  );

  Future<core.WalletState> getWalletState() => call(core.getWalletState);

  Future<String> getRegistrationRevocationCode() => _callWithFlowBreadcrumb(
    'registration_revocation_code.get',
    failureCode: 'registration_revocation_code.fail.get',
    runnable: core.getRegistrationRevocationCode,
  );

  Future<core.RevocationCodeResult> getRevocationCode(String pin) => _callWithFlowBreadcrumb(
    'revocation_code.get',
    failureCode: 'revocation_code.fail.get',
    runnable: () => core.getRevocationCode(pin: pin),
  );

  /// This function should be used to call through to the core, as it makes sure potential exceptions are processed
  /// before they are (re)thrown.
  Future<T> call<T>(Future<T> Function() runnable) async {
    try {
      return await runnable();
    } catch (exception, stacktrace) {
      throw await _handleCoreException(exception, stackTrace: stacktrace);
    }
  }

  Future<T> _callWithFlowBreadcrumb<T>(
    String code, {
    required String failureCode,
    required Future<T> Function() runnable,
  }) async {
    await SentryBreadcrumbs.flow(code);
    try {
      return await call(runnable);
    } catch (_) {
      await SentryBreadcrumbs.flow(failureCode);
      rethrow;
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
        // Invariant violation (programming error / panic). Report it, then fall through so the error
        // listener can navigate to the invariant error screen instead of crashing the app.
        Fimber.e(
          'StateError detected, this indicates a programming error.',
          ex: error,
          stacktrace: stackTrace,
        );
        await Sentry.captureException(error, stackTrace: stackTrace);
      }
      // Use microtask so the listener triggers after the error is returned. (crucial to allow downstream navigation)
      unawaited(Future.microtask(() => _errorListener?.call(error)));
      return error;
    } catch (exception) {
      Fimber.e(
        'Failed to map exception ($ex) to CoreError, returning original exception',
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
