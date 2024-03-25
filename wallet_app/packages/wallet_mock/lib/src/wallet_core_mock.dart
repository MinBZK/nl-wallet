import 'dart:convert';

import 'package:wallet_core/core.dart';

import '../mock.dart';
import 'data/mock/mock_cards.dart';
import 'data/mock/mock_disclosure_requests.dart';
import 'log/wallet_event_log.dart';
import 'pin/pin_manager.dart';
import 'util/extension/string_extension.dart';
import 'wallet/wallet.dart';

class WalletCoreMock extends _FlutterRustBridgeTasksMeta implements WalletCore {
  bool _isInitialized = false;
  StartDisclosureResult? _ongoingDisclosure;

  final PinManager _pinManager;
  final Wallet _wallet;
  final WalletEventLog _eventLog;

  WalletCoreMock(this._pinManager, this._wallet, this._eventLog);

  @override
  Future<StartDisclosureResult> startDisclosure({required String uri, hint}) async {
    // Look up the associated request
    final jsonPayload = jsonDecode(Uri.decodeComponent(Uri.parse(uri).fragment));
    final disclosureId = jsonPayload['id'] as String;
    final request = kDisclosureRequests.firstWhere((element) => element.id == disclosureId);
    final requestOriginBaseUrl = request.relyingParty.webUrl ?? 'http://origin.org';

    // Check if all attributes are available
    final containsAllRequestedAttributes =
        _wallet.containsAttributes(request.requestedAttributes.map((requestedAttribute) => requestedAttribute.key));
    if (containsAllRequestedAttributes) {
      return _ongoingDisclosure = StartDisclosureResult.request(
        relyingParty: request.relyingParty,
        policy: request.policy,
        requestedCards: _wallet.getDisclosureCards(request.requestedAttributes.map((attribute) => attribute.key)),
        sharedDataWithRelyingPartyBefore: _eventLog.includesInteractionWith(request.relyingParty),
        sessionType: DisclosureSessionType.CrossDevice,
        requestOriginBaseUrl: requestOriginBaseUrl,
        requestPurpose: request.purpose.untranslated,
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
  Future<void> cancelDisclosure({hint}) async {
    final disclosure = _ongoingDisclosure;
    assert(disclosure != null, 'No ongoing disclosure to deny');
    _ongoingDisclosure = null;
    _eventLog.logDisclosure(disclosure!, DisclosureStatus.Cancelled);
  }

  @override
  Future<AcceptDisclosureResult> acceptDisclosure({required String pin, hint}) async {
    final disclosure = _ongoingDisclosure;
    assert(disclosure != null, 'No ongoing disclosure to accept');
    assert(disclosure is StartDisclosureResult_Request, 'Can\'t accept disclosure with missing attributes');

    // Check if correct pin was provided
    final result = _pinManager.checkPin(pin);
    if (result is WalletInstructionResult_InstructionError) {
      return AcceptDisclosureResult.instructionError(error: result.error);
    }

    // Log successful disclosure
    _eventLog.logDisclosure(disclosure!, DisclosureStatus.Success);
    _ongoingDisclosure = null;

    return AcceptDisclosureResult.ok();
  }

  @override
  Future<WalletInstructionResult> acceptPidIssuance({required String pin, hint}) async {
    final result = _pinManager.checkPin(pin);
    if (result is! WalletInstructionResult_Ok) return result;

    assert(_wallet.isEmpty, 'We can only accept the pid if the wallet was previously empty');
    // Add the PID cards to the user's wallet
    _wallet.add(kPidCards);
    // Log the issuance events
    for (final card in kPidCards) {
      _eventLog.logIssuance(card);
    }

    return result;
  }

  @override
  Future<void> cancelPidIssuance({hint}) async {
    // Stub only, no need to cancel it on the mock
  }

  @override
  Future<void> clearCardsStream({hint}) async {
    // Stub only, no need to clear it on the mock
  }

  @override
  Future<void> clearConfigurationStream({hint}) async {
    // Stub only, no need to clear it on the mock
  }

  @override
  Future<void> clearLockStream({hint}) async {
    // Stub only, no need to clear it on the mock
  }

  @override
  Future<void> clearRecentHistoryStream({hint}) async {
    // Stub only, no need to clear it on the mock
  }

  @override
  Future<List<Card>> continuePidIssuance({required String uri, hint}) async => kPidCards;

  @override
  Future<String> createPidIssuanceRedirectUri({hint}) async => kMockPidIssuanceRedirectUri;

  @override
  Future<bool> hasRegistration({hint}) async => _pinManager.isRegistered;

  @override
  Future<IdentifyUriResult> identifyUri({required String uri, hint}) async {
    final jsonPayload = jsonDecode(Uri.decodeComponent(Uri.parse(uri).fragment));
    final type = jsonPayload['type'] as String;
    if (type == 'verify') return IdentifyUriResult.Disclosure;
    if (type == 'issue') throw UnsupportedError('Issue not yet supported');
    if (type == 'sign') throw UnsupportedError('Sign not yet supported');
    throw UnsupportedError('Unsupported uri: $uri');
  }

  @override
  Future<void> init({hint}) async => _isInitialized = true;

  @override
  Future<bool> isInitialized({hint}) async => _isInitialized;

  @override
  Future<PinValidationResult> isValidPin({required String pin, hint}) async {
    const digits = 6;
    if (pin.length != digits) return PinValidationResult.OtherIssue;
    if (pin.split('').toSet().length <= 1) return PinValidationResult.TooFewUniqueDigits;

    // Check for ascending or descending sequences
    final pinDigits = pin.split('').map((e) => int.parse(e));
    var ascending = true;
    var descending = true;
    int? prev;
    for (var digit in pinDigits) {
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
  Future<void> lockWallet({hint}) async => _wallet.lock();

  @override
  Future<void> register({required String pin, hint}) async {
    _pinManager.setPin(pin);
    _wallet.unlock();
  }

  @override
  Future<void> resetWallet({hint}) async {
    _pinManager.resetPin();
    _wallet.reset();
    _eventLog.reset();
  }

  @override
  Stream<List<Card>> setCardsStream({hint}) => _wallet.cardsStream;

  @override
  Stream<FlutterConfiguration> setConfigurationStream({hint}) {
    return Stream.value(
      FlutterConfiguration(
        backgroundLockTimeout: Duration(minutes: 5).inSeconds,
        inactiveLockTimeout: Duration(minutes: 20).inSeconds,
        version: 1,
      ),
    );
  }

  @override
  Stream<bool> setLockStream({hint}) => _wallet.lockedStream;

  @override
  Future<WalletInstructionResult> unlockWallet({required String pin, hint}) async {
    final result = _pinManager.checkPin(pin);
    bool pinMatches = result is WalletInstructionResult_Ok;
    if (pinMatches) {
      _wallet.unlock();
    } else {
      _wallet.lock();
    }
    return result;
  }

  @override
  Future<List<WalletEvent>> getHistory({hint}) async => _eventLog.log;

  @override
  Future<List<WalletEvent>> getHistoryForCard({required String docType, hint}) async =>
      _eventLog.logForDocType(docType);

  @override
  Stream<List<WalletEvent>> setRecentHistoryStream({hint}) => _eventLog.logStream;

  @override
  Future<bool> hasActiveDisclosureSession({hint}) async => _ongoingDisclosure != null;

  @override
  Future<bool> hasActivePidIssuanceSession({hint}) async => false;
}

/// Helper class to make [WalletCoreMock] satisfy [WalletCore]
/// without cluttering it with getters we don't intend to implement.
class _FlutterRustBridgeTasksMeta {
  FlutterRustBridgeTaskConstMeta get kAcceptDisclosureConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kAcceptPidIssuanceConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kCancelDisclosureConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kCancelPidIssuanceConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kClearCardsStreamConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kClearConfigurationStreamConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kClearLockStreamConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kContinuePidIssuanceConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kCreatePidIssuanceRedirectUriConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kHasRegistrationConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kIdentifyUriConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kInitConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kIsInitializedConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kIsValidPinConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kLockWalletConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kRegisterConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kRejectPidIssuanceConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kResetWalletConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kSetCardsStreamConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kSetConfigurationStreamConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kSetLockStreamConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kStartDisclosureConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kUnlockWalletConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kGetHistoryConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kGetHistoryForCardConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kClearRecentHistoryStreamConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kSetRecentHistoryStreamConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kHasActiveDisclosureSessionConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kHasActivePidIssuanceSessionConstMeta => throw UnimplementedError();
}
