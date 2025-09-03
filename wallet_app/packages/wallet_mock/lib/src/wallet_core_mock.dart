import 'dart:convert';

import 'package:wallet_core/core.dart';

import '../mock.dart';
import 'data/mock/mock_attestations.dart';
import 'disclosure_manager.dart';
import 'log/wallet_event_log.dart';
import 'pin/pin_manager.dart';
import 'wallet/wallet.dart';

class WalletCoreMock implements WalletCoreApi {
  /// Simulate the behaviour of the real WalletCore, which requires a call to `init`
  bool _isInitialized = false;

  final IssuanceManager _issuanceManager;
  final DisclosureManager _disclosureManager;

  final PinManager _pinManager;
  final Wallet _wallet;
  final WalletEventLog _eventLog;

  bool _isBiometricsEnabled = false;

  WalletCoreMock(this._pinManager, this._wallet, this._eventLog, this._issuanceManager, this._disclosureManager);

  @override
  Future<StartDisclosureResult> crateApiFullStartDisclosure({required String uri, required bool isQrCode}) async {
    final jsonPayload = jsonDecode(Uri.decodeComponent(Uri.parse(uri).fragment));
    final isDisclosureBasedIssuance = jsonPayload['type'] == 'issue';

    // Detect and re-route disclosure based issuance requests
    if (isDisclosureBasedIssuance) return _issuanceManager.startIssuance(uri);
    // Proceed with normal disclosure
    return _disclosureManager.startDisclosure(uri, isQrCode: isQrCode);
  }

  @override
  Future<String?> crateApiFullCancelDisclosure() async {
    await _disclosureManager.cancelDisclosure();
    return null;
  }

  @override
  Future<AcceptDisclosureResult> crateApiFullAcceptDisclosure({required String pin}) async {
    return _disclosureManager.acceptDisclosure(pin);
  }

  @override
  Future<WalletInstructionResult> crateApiFullAcceptIssuance({required String pin}) async {
    /// Check if the issuance manager has an active session that should be continued
    if (_issuanceManager.hasActiveIssuanceSession) return _issuanceManager.acceptIssuance(pin, []);

    /// Validate PIN
    final result = _pinManager.checkPin(pin);

    /// Check if PID is already in wallet (renewal flow)
    if (!_wallet.isEmpty) {
      // Not empty so user must have PID, thus trigger renewal.
      if (result is! WalletInstructionResult_Ok) return result;
      _wallet.add(kPidAttestations);
      for (final it in kPidAttestations) {
        _eventLog.logIssuance(it, isRenewal: true);
      }
      return result;
    }

    /// Continue with PID issuance flow
    if (result is WalletInstructionResult_InstructionError && result.error is WalletInstructionError_Timeout) {
      /// PVW-1037 (criteria 6): Handle the special case where the user has forgotten her pin during initial setup.
      await resetWallet();
    }
    if (result is! WalletInstructionResult_Ok) return result;

    // Add the PID cards to the user's wallet
    _wallet.add(kPidAttestations);
    // Log the issuance events
    kPidAttestations.forEach(_eventLog.logIssuance);
    return result;
  }

  @override
  Future<PidIssuanceResult> crateApiFullAcceptPidIssuance({required String pin}) {
    throw UnimplementedError();
  }

  @override
  Future<void> crateApiFullCancelIssuance() async => _issuanceManager.cancelIssuance();

  @override
  Future<void> crateApiFullClearAttestationsStream() async {
    // Stub only, no need to clear it on the mock
  }

  @override
  Future<void> crateApiFullClearConfigurationStream() async {
    // Stub only, no need to clear it on the mock
  }

  @override
  Future<void> crateApiFullClearVersionStateStream() async {
    // Stub only, no need to clear it on the mock
  }

  @override
  Future<void> crateApiFullClearLockStream() async {
    // Stub only, no need to clear it on the mock
  }

  @override
  Future<void> crateApiFullClearRecentHistoryStream() async {
    // Stub only, no need to clear it on the mock
  }

  @override
  Future<List<AttestationPresentation>> crateApiFullContinuePidIssuance({required String uri}) async =>
      kPidAttestations;

  @override
  Future<String> crateApiFullCreatePidIssuanceRedirectUri() async => MockConstants.pidIssuanceRedirectUri;

  @override
  Future<bool> crateApiFullHasRegistration() async => _pinManager.isRegistered;

  @override
  Future<IdentifyUriResult> crateApiFullIdentifyUri({required String uri}) async {
    final jsonPayload = jsonDecode(Uri.decodeComponent(Uri.parse(uri).fragment));
    final type = jsonPayload['type'] as String;
    if (type == 'verify') return IdentifyUriResult.Disclosure;
    if (type == 'issue') return IdentifyUriResult.DisclosureBasedIssuance;
    if (type == 'sign') throw UnsupportedError('Sign not yet supported');
    throw UnsupportedError('Unsupported uri: $uri');
  }

  @override
  Future<void> crateApiFullInit() async => _isInitialized = true;

  @override
  Future<bool> crateApiFullIsInitialized() async => _isInitialized;

  @override
  Future<PinValidationResult> crateApiFullIsValidPin({required String pin}) async {
    const digits = 6;
    if (pin.length != digits) return PinValidationResult.OtherIssue;
    if (pin.split('').toSet().length <= 1) return PinValidationResult.TooFewUniqueDigits;

    // Check for ascending or descending sequences
    final pinDigits = pin.split('').map(int.parse);
    var ascending = true;
    var descending = true;
    int? prev;
    for (final digit in pinDigits) {
      if (prev != null) {
        if (digit != prev + 1) ascending = false;
        if (digit != prev - 1) descending = false;
      }
      prev = digit;
    }
    if (ascending || descending) return PinValidationResult.SequentialDigits;

    return PinValidationResult.Ok;
  }

  @override
  Future<void> crateApiFullLockWallet() async => _wallet.lock();

  @override
  Future<void> crateApiFullRegister({required String pin}) async {
    _pinManager.setPin(pin);
    _wallet.unlock();
  }

  @override
  Future<void> crateApiFullResetWallet() async {
    await _pinManager.resetPin();
    _wallet.reset();
    _eventLog.reset();
  }

  @override
  Stream<List<AttestationPresentation>> crateApiFullSetAttestationsStream() => _wallet.attestationsStream;

  @override
  Stream<FlutterConfiguration> crateApiFullSetConfigurationStream() {
    return Stream.value(
      FlutterConfiguration(
        inactiveWarningTimeout: const Duration(minutes: 1).inSeconds,
        inactiveLockTimeout: const Duration(minutes: 3).inSeconds,
        backgroundLockTimeout: const Duration(seconds: 20).inSeconds,
        staticAssetsBaseUrl: 'https://example.com/',
        version: BigInt.one,
      ),
    );
  }

  @override
  Stream<FlutterVersionState> crateApiFullSetVersionStateStream() {
    return Stream.value(const FlutterVersionState.ok());
  }

  @override
  Stream<bool> crateApiFullSetLockStream() => _wallet.lockedStream;

  @override
  Future<WalletInstructionResult> crateApiFullUnlockWallet({required String pin}) async {
    final result = _pinManager.checkPin(pin);
    final bool pinMatches = result is WalletInstructionResult_Ok;
    if (pinMatches) {
      _wallet.unlock();
    } else {
      _wallet.lock();
    }
    return result;
  }

  @override
  Future<List<WalletEvent>> crateApiFullGetHistory() async => _eventLog.log;

  @override
  Future<List<WalletEvent>> crateApiFullGetHistoryForCard({required String attestationId}) async =>
      _eventLog.logForAttestationId(attestationId);

  @override
  Stream<List<WalletEvent>> crateApiFullSetRecentHistoryStream() => _eventLog.logStream;

  @override
  Future<bool> crateApiFullHasActiveDisclosureSession() async => _disclosureManager.hasActiveDisclosureSession;

  @override
  Future<bool> crateApiFullHasActiveIssuanceSession() async => _issuanceManager.hasActiveIssuanceSession;

  @override
  Future<DisclosureBasedIssuanceResult> crateApiFullContinueDisclosureBasedIssuance({required String pin}) async {
    assert(_issuanceManager.hasActiveIssuanceSession, 'invalid state');
    final attestations = await _issuanceManager.discloseForIssuance(pin);
    try {
      return DisclosureBasedIssuanceResult.ok(attestations);
    } on WalletInstructionError catch (error) {
      return DisclosureBasedIssuanceResult.instructionError(error: error);
    } catch (ex) {
      rethrow;
    }
  }

  @override
  Future<bool> crateApiFullIsBiometricUnlockEnabled() async => _isBiometricsEnabled;

  @override
  Future<void> crateApiFullUnlockWalletWithBiometrics() async => _wallet.unlock();

  @override
  Future<void> crateApiFullSetBiometricUnlock({required bool enable}) async => _isBiometricsEnabled = enable;

  @override
  Future<WalletInstructionResult> crateApiFullChangePin({required String oldPin, required String newPin}) async {
    final result = _pinManager.checkPin(oldPin);
    final validationResult = await isValidPin(pin: newPin);
    if (validationResult != PinValidationResult.Ok) throw StateError('Pin should be validated in the flow beforehand');
    await Future.delayed(const Duration(seconds: 1));
    // Should be followed with a call to [continueChangePin] to actually update the PIN.
    return result;
  }

  @override
  Future<WalletInstructionResult> crateApiFullContinueChangePin({required String pin}) async {
    _pinManager.updatePin(pin);
    await Future.delayed(const Duration(milliseconds: 500));
    return _pinManager.checkPin(pin);
  }

  @override
  Future<WalletInstructionResult> crateApiFullCheckPin({required String pin}) async => _pinManager.checkPin(pin);

  @override
  Future<String> crateApiFullGetVersionString() async => MockConstants.versionString;

  @override
  Future<String> crateApiFullCreatePidRenewalRedirectUri() async {
    await Future.delayed(const Duration(seconds: 1));
    return MockConstants.pidRenewalRedirectUri;
  }

  @override
  Future<String> crateApiFullCreatePinRecoveryRedirectUri() {
    throw UnimplementedError();
  }

  @override
  Future<void> crateApiFullContinuePinRecovery({required String uri}) {
    throw UnimplementedError();
  }

  @override
  Future<WalletInstructionResult> crateApiFullCompletePinRecovery({required String pin}) {
    throw UnimplementedError();
  }

  @override
  Future<void> crateApiFullCancelPinRecovery() {
    throw UnimplementedError();
  }

  @override
  Future<void> crateApiFullInitWalletTransfer({required String uri}) {
    throw UnimplementedError();
  }

  @override
  Future<WalletInstructionResult> crateApiFullConfirmWalletTransfer({required String pin}) {
    throw UnimplementedError();
  }

  @override
  Future<WalletTransferInstructionResult> crateApiFullStartWalletTransfer({required String pin}) {
    throw UnimplementedError();
  }

  @override
  Stream<WalletTransferState> crateApiFullGetWalletTransferStateStream() {
    throw UnimplementedError();
  }
}
