import 'dart:convert';

import 'package:wallet_core/core.dart';

import 'data/mock/mock_sign_requests.dart';
import 'data/model/sign_request.dart';
import 'log/wallet_event_log.dart';
import 'pin/pin_manager.dart';
import 'util/extension/string_extension.dart';
import 'wallet/wallet.dart';

export 'data/model/sign_request.dart' show Document;

/// Since the core doesn't support signing yet, this class defines a sign
/// api which closely resembles the disclosure flow. Once [WalletCore] does support
/// signing the the mock can be implemented there (through [WalletCoreMock]) and this
/// class should be deleted.
class WalletCoreForSigning {
  final PinManager _pinManager;
  final Wallet _wallet;
  final WalletEventLog _eventLog;

  SignRequest? _activeSignRequest;

  /// Get the cards/attributes that have to be disclosed to fulfill [_activeSignRequest], assumes [_activeSignRequest] is non null.
  List<AttestationPresentation> get _requestedAttestationsForActiveRequest => _wallet.getRequestedAttestations(
        _activeSignRequest!.requestedAttributes.map(
          (attribute) => attribute.key,
        ),
      );

  WalletCoreForSigning(this._pinManager, this._wallet, this._eventLog);

  Future<StartSigningResult> startSigning(String uri) async {
    // Look up the associated response
    final jsonPayload = jsonDecode(Uri.decodeComponent(Uri.parse(uri).fragment));
    final signId = jsonPayload['id'] as String;
    final request = _activeSignRequest = kSignRequests.firstWhere((element) => element.id == signId);

    final signPossible = _wallet.containsAttributes(request.requestedAttributes.map((e) => e.key));
    if (signPossible) {
      return StartSignResultReadyToDisclose(
        organization: request.organization,
        policy: request.policy,
        trustProvider: request.trustProvider,
        document: request.document,
        requestedAttestations: _requestedAttestationsForActiveRequest,
      );
    } else {
      final requestedAttributesNotInWallet =
          _wallet.getMissingAttributeKeys(request.requestedAttributes.map((e) => e.key));
      final missingAttributes = requestedAttributesNotInWallet.map((key) {
        final associatedLabel = request.requestedAttributes.firstWhere((element) => element.key == key).label;
        return MissingAttribute(labels: associatedLabel.untranslated);
      });
      return StartSignResultRequestedAttributesMissing(
        organization: request.organization,
        policy: request.policy,
        trustProvider: request.trustProvider,
        document: request.document,
        missingAttributes: missingAttributes.toList(),
      );
    }
  }

  Future<WalletInstructionResult> signAgreement(String pin) async {
    assert(_activeSignRequest != null, 'Can not sign when no sign is active');
    final result = _pinManager.checkPin(pin);
    if (result is WalletInstructionResult_Ok) {
      _eventLog.logDisclosureStep(
        _activeSignRequest!.organization,
        _activeSignRequest!.policy,
        _requestedAttestationsForActiveRequest,
        DisclosureStatus.Success,
        purpose: [
          const LocalizedString(language: 'en', value: 'Signing'),
          const LocalizedString(language: 'en', value: 'Ondertekenen'),
        ],
      );
    }
    return result;
  }

  Future<void> rejectAgreement() async {
    if (_activeSignRequest != null) {
      _eventLog.logDisclosureStep(
        _activeSignRequest!.organization,
        _activeSignRequest!.policy,
        _requestedAttestationsForActiveRequest,
        DisclosureStatus.Cancelled,
        purpose: [
          const LocalizedString(language: 'en', value: 'Signing'),
          const LocalizedString(language: 'en', value: 'Ondertekenen'),
        ],
      );
    }
    _activeSignRequest = null;
  }
}

sealed class StartSigningResult {
  final Organization organization;
  final Organization trustProvider;
  final Document document;
  final RequestPolicy policy;

  StartSigningResult({
    required this.organization,
    required this.policy,
    required this.trustProvider,
    required this.document,
  });
}

class StartSignResultReadyToDisclose extends StartSigningResult {
  final List<AttestationPresentation> requestedAttestations;

  StartSignResultReadyToDisclose({
    required super.organization,
    required super.policy,
    required super.trustProvider,
    required super.document,
    required this.requestedAttestations,
  });
}

class StartSignResultRequestedAttributesMissing extends StartSigningResult {
  final List<MissingAttribute> missingAttributes;

  StartSignResultRequestedAttributesMissing({
    required super.organization,
    required super.policy,
    required super.trustProvider,
    required super.document,
    required this.missingAttributes,
  });
}
