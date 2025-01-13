import 'dart:convert';

import 'package:wallet_core/core.dart';

import '../mock.dart';
import 'data/mock/mock_cards.dart';
import 'data/mock/mock_disclosure_requests.dart';
import 'log/wallet_event_log.dart';
import 'pin/pin_manager.dart';
import 'util/extension/string_extension.dart';
import 'wallet/wallet.dart';

class WalletCoreMock implements WalletCoreApi {
  bool _isInitialized = false;
  StartDisclosureResult? _ongoingDisclosure;

  final PinManager _pinManager;
  final Wallet _wallet;
  final WalletEventLog _eventLog;
  bool _isBiometricsEnabled = false;

  WalletCoreMock(this._pinManager, this._wallet, this._eventLog);

  @override
  Future<StartDisclosureResult> crateApiFullStartDisclosure({required String uri, required bool isQrCode, hint}) async {
    // Look up the associated request
    final jsonPayload = jsonDecode(Uri.decodeComponent(Uri.parse(uri).fragment));
    final disclosureId = jsonPayload['id'] as String;
    final request = kDisclosureRequests.firstWhere((element) => element.id == disclosureId);
    final requestOriginBaseUrl = request.relyingParty.webUrl ?? 'http://origin.org';

    // Check if all attributes are available
    final containsAllRequestedAttributes =
        _wallet.containsAttributes(request.requestedAttributes.map((requestedAttribute) => requestedAttribute.key));
    if (containsAllRequestedAttributes) {
      final isLoginRequest =
          request.requestedAttributes.length == 1 && request.requestedAttributes.first.key == 'mock.citizenshipNumber';
      return _ongoingDisclosure = StartDisclosureResult.request(
        relyingParty: request.relyingParty,
        policy: request.policy,
        requestedCards: _wallet.getDisclosureCards(request.requestedAttributes.map((attribute) => attribute.key)),
        sharedDataWithRelyingPartyBefore: _eventLog.includesInteractionWith(request.relyingParty),
        sessionType: DisclosureSessionType.CrossDevice,
        requestOriginBaseUrl: requestOriginBaseUrl,
        requestPurpose: request.purpose.untranslated,
        requestType: isLoginRequest ? DisclosureType.Login : DisclosureType.Regular,
      );
    } else {
      final requestedAttributesNotInWallet =
          _wallet.getMissingAttributeKeys(request.requestedAttributes.map((e) => e.key));
      final missingAttributes = requestedAttributesNotInWallet.map((key) {
        final associatedLabel = request.requestedAttributes.firstWhere((element) => element.key == key).label;
        return MissingAttribute(labels: associatedLabel.untranslated);
      });
      return _ongoingDisclosure = StartDisclosureResult.requestAttributesMissing(
        relyingParty: request.relyingParty,
        sharedDataWithRelyingPartyBefore: _eventLog.includesInteractionWith(request.relyingParty),
        sessionType: DisclosureSessionType.CrossDevice,
        requestOriginBaseUrl: requestOriginBaseUrl,
        requestPurpose: request.purpose.untranslated,
        missingAttributes: missingAttributes.toList(),
      );
    }
  }

  @override
  Future<String?> crateApiFullCancelDisclosure({hint}) async {
    final disclosure = _ongoingDisclosure;
    assert(disclosure != null, 'No ongoing disclosure to deny');
    _ongoingDisclosure = null;
    _eventLog.logDisclosure(disclosure!, DisclosureStatus.Cancelled);
    return null;
  }

  @override
  Future<AcceptDisclosureResult> crateApiFullAcceptDisclosure({required String pin, hint}) async {
    final disclosure = _ongoingDisclosure;
    assert(disclosure != null, 'No ongoing disclosure to accept');
    assert(disclosure is StartDisclosureResult_Request, "Can't accept disclosure with missing attributes");

    // Check if correct pin was provided
    final result = _pinManager.checkPin(pin);
    if (result is WalletInstructionResult_InstructionError) {
      switch (result.error) {
        case WalletInstructionError_Timeout():
        case WalletInstructionError_Blocked():
          _wallet.lock();
        case WalletInstructionError_IncorrectPin():
        // TODO: Handle this case.
      }
      return AcceptDisclosureResult.instructionError(error: result.error);
    }

    // Log successful disclosure
    _eventLog.logDisclosure(disclosure!, DisclosureStatus.Success);
    _ongoingDisclosure = null;

    return AcceptDisclosureResult.ok();
  }

  @override
  Future<WalletInstructionResult> crateApiFullAcceptPidIssuance({required String pin, hint}) async {
    final result = _pinManager.checkPin(pin);
    if (result is WalletInstructionResult_InstructionError && result.error is WalletInstructionError_Timeout) {
      /// PVW-1037 (criteria 6): Handle the special case where the user has forgotten her pin during initial setup.
      await resetWallet();
    }
    if (result is! WalletInstructionResult_Ok) return result;

    assert(_wallet.isEmpty, 'We can only accept the pid if the wallet was previously empty');
    // Add the PID cards to the user's wallet
    _wallet.add(kPidCards);
    // Log the issuance events
    kPidCards.forEach(_eventLog.logIssuance);
    return result;
  }

  @override
  Future<void> crateApiFullCancelPidIssuance({hint}) async {
    // Stub only, no need to cancel it on the mock
  }

  @override
  Future<void> crateApiFullClearCardsStream({hint}) async {
    // Stub only, no need to clear it on the mock
  }

  @override
  Future<void> crateApiFullClearConfigurationStream({hint}) async {
    // Stub only, no need to clear it on the mock
  }

  @override
  Future<void> crateApiFullClearVersionStateStream({hint}) async {
    // Stub only, no need to clear it on the mock
  }

  @override
  Future<void> crateApiFullClearLockStream({hint}) async {
    // Stub only, no need to clear it on the mock
  }

  @override
  Future<void> crateApiFullClearRecentHistoryStream({hint}) async {
    // Stub only, no need to clear it on the mock
  }

  @override
  Future<List<Card>> crateApiFullContinuePidIssuance({required String uri, hint}) async => kPidCards;

  @override
  Future<String> crateApiFullCreatePidIssuanceRedirectUri({hint}) async => kMockPidIssuanceRedirectUri;

  @override
  Future<bool> crateApiFullHasRegistration({hint}) async => _pinManager.isRegistered;

  @override
  Future<IdentifyUriResult> crateApiFullIdentifyUri({required String uri, hint}) async {
    final jsonPayload = jsonDecode(Uri.decodeComponent(Uri.parse(uri).fragment));
    final type = jsonPayload['type'] as String;
    if (type == 'verify') return IdentifyUriResult.Disclosure;
    if (type == 'issue') throw UnsupportedError('Issue not yet supported');
    if (type == 'sign') throw UnsupportedError('Sign not yet supported');
    throw UnsupportedError('Unsupported uri: $uri');
  }

  @override
  Future<void> crateApiFullInit({hint}) async => _isInitialized = true;

  @override
  Future<bool> crateApiFullIsInitialized({hint}) async => _isInitialized;

  @override
  Future<PinValidationResult> crateApiFullIsValidPin({required String pin, hint}) async {
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
  Future<void> crateApiFullLockWallet({hint}) async => _wallet.lock();

  @override
  Future<void> crateApiFullRegister({required String pin, hint}) async {
    _pinManager.setPin(pin);
    _wallet.unlock();
  }

  @override
  Future<void> crateApiFullResetWallet({hint}) async {
    await _pinManager.resetPin();
    _wallet.reset();
    _eventLog.reset();
  }

  @override
  Stream<List<Card>> crateApiFullSetCardsStream({hint}) => _wallet.cardsStream;

  @override
  Stream<FlutterConfiguration> crateApiFullSetConfigurationStream({hint}) {
    return Stream.value(
      FlutterConfiguration(
        backgroundLockTimeout: Duration(seconds: 20).inSeconds,
        inactiveLockTimeout: Duration(minutes: 3).inSeconds,
        version: 1,
      ),
    );
  }

  @override
  Stream<FlutterVersionState> crateApiFullSetVersionStateStream({hint}) {
    return Stream.value(FlutterVersionState.ok());
  }

  @override
  Stream<bool> crateApiFullSetLockStream({hint}) => _wallet.lockedStream;

  @override
  Future<WalletInstructionResult> crateApiFullUnlockWallet({required String pin, hint}) async {
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
  Future<List<WalletEvent>> crateApiFullGetHistory({hint}) async => _eventLog.log;

  @override
  Future<List<WalletEvent>> crateApiFullGetHistoryForCard({required String docType, hint}) async =>
      _eventLog.logForDocType(docType);

  @override
  Stream<List<WalletEvent>> crateApiFullSetRecentHistoryStream({hint}) => _eventLog.logStream;

  @override
  Future<bool> crateApiFullHasActiveDisclosureSession({hint}) async => _ongoingDisclosure != null;

  @override
  Future<bool> crateApiFullHasActivePidIssuanceSession({hint}) async => false;

  @override
  Future<bool> crateApiFullIsBiometricUnlockEnabled({hint}) async => _isBiometricsEnabled;

  @override
  Future<void> crateApiFullUnlockWalletWithBiometrics({hint}) async => _wallet.unlock();

  @override
  Future<void> crateApiFullSetBiometricUnlock({required bool enable, hint}) async => _isBiometricsEnabled = enable;

  @override
  Future<WalletInstructionResult> crateApiFullChangePin({required String oldPin, required String newPin, hint}) async {
    final result = _pinManager.checkPin(oldPin);
    final validationResult = await isValidPin(pin: newPin);
    if (validationResult != PinValidationResult.Ok) throw StateError('Pin should be validated in the flow beforehand');
    await Future.delayed(const Duration(seconds: 1));
    // Should be followed with a call to [continueChangePin] to actually update the PIN.
    return result;
  }

  @override
  Future<WalletInstructionResult> crateApiFullContinueChangePin({required String pin, hint}) async {
    _pinManager.updatePin(pin);
    await Future.delayed(const Duration(milliseconds: 500));
    return _pinManager.checkPin(pin);
  }

  @override
  Future<WalletInstructionResult> crateApiFullCheckPin({required String pin, hint}) async => _pinManager.checkPin(pin);

  @override
  Future<String> crateApiFullGetVersionString({hint}) async => kMockVersionString;
}
