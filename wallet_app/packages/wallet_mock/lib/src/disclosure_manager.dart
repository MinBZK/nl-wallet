import 'dart:convert';

import 'package:wallet_core/core.dart';

import 'data/mock/mock_disclosure_requests.dart';
import 'log/wallet_event_log.dart';
import 'pin/pin_manager.dart';
import 'util/extension/string_extension.dart';
import 'wallet/wallet.dart';

/// This class manages (mock) disclosure sessions. Previously this was part of the
/// [WalletCoreMock] directly but moved for better separation of concerns.
class DisclosureManager {
  final PinManager _pinManager;
  final Wallet _wallet;
  final WalletEventLog _eventLog;

  StartDisclosureResult? _ongoingDisclosure;

  bool get hasActiveDisclosureSession => _ongoingDisclosure != null;

  DisclosureManager(this._pinManager, this._wallet, this._eventLog);

  Future<StartDisclosureResult> startDisclosure(String uri, {required bool isQrCode}) async {
    final jsonPayload = jsonDecode(Uri.decodeComponent(Uri.parse(uri).fragment));

    // Look up the associated request
    final disclosureId = jsonPayload['id'] as String;
    final request = kDisclosureRequests.firstWhere((element) => element.id == disclosureId);
    final requestOriginBaseUrl = request.relyingParty.webUrl ?? 'http://origin.org';

    // Check if all attributes are available
    final containsAllRequestedAttributes =
        _wallet.containsAttributes(request.requestedAttributes.map((requestedAttribute) => requestedAttribute.key));
    if (containsAllRequestedAttributes) {
      final isLoginRequest =
          request.requestedAttributes.length == 1 && request.requestedAttributes.first.key == 'mock_citizenshipNumber';
      return _ongoingDisclosure = StartDisclosureResult.request(
        relyingParty: request.relyingParty,
        policy: request.policy,
        requestedAttestations:
            _wallet.getRequestedAttestations(request.requestedAttributes.map((attribute) => attribute.key)),
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

  Future<void> cancelDisclosure() async {
    final disclosure = _ongoingDisclosure;
    assert(disclosure != null, 'No ongoing disclosure to deny');
    _ongoingDisclosure = null;
    _eventLog.logDisclosure(disclosure!, DisclosureStatus.Cancelled);
  }

  Future<AcceptDisclosureResult> acceptDisclosure(String pin) async {
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
      }
      return AcceptDisclosureResult.instructionError(error: result.error);
    }

    // Log successful disclosure
    _eventLog.logDisclosure(disclosure!, DisclosureStatus.Success);
    _ongoingDisclosure = null;

    return AcceptDisclosureResult.ok();
  }
}
