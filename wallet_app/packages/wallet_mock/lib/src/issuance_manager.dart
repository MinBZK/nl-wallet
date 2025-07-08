import 'dart:convert';

import 'package:wallet_core/core.dart';

import 'data/mock/mock_issuance_responses.dart';
import 'data/model/issuance_response.dart';
import 'log/wallet_event_log.dart';
import 'pin/pin_manager.dart';
import 'util/extension/string_extension.dart';
import 'wallet/wallet.dart';

/// This class manages 'disclosure based issuance' requests. It was initially
/// a class that exposed an API directly to the user (wallet_app) but since the
/// core api now supports disclosure based issuance this has been turned into
/// a 'manager' to wrap the mock logic to support the issuance flows. See
/// [WalletCoreMock]'s issuance related methods.
class IssuanceManager {
  final PinManager _pinManager;
  final Wallet _wallet;
  final WalletEventLog _eventLog;

  IssuanceResponse? _activeIssuanceResponse;
  bool _itemsHaveBeenDisclosed = false;

  bool get hasActiveIssuanceSession => _activeIssuanceResponse != null;

  /// Get the cards/attributes that have to be disclosed to fulfill [_activeIssuanceResponse], assumes [_activeIssuanceResponse] is non null.
  List<AttestationPresentation> get _requestedAttestationsForActiveRequest => _wallet.getRequestedAttestations(
        _activeIssuanceResponse!.requestedAttributes.map(
          (attribute) => attribute.key,
        ),
      );

  IssuanceManager(this._pinManager, this._wallet, this._eventLog);

  Future<StartDisclosureResult> startIssuance(String uri) async {
    // Look up the associated response
    final jsonPayload = jsonDecode(Uri.decodeComponent(Uri.parse(uri).fragment));
    final issuanceId = jsonPayload['id'] as String;
    final response = _activeIssuanceResponse = kIssuanceResponses.firstWhere((element) => element.id == issuanceId);

    final issuancePossible = _wallet.containsAttributes(response.requestedAttributes.map((e) => e.key));
    if (issuancePossible) {
      return StartDisclosureResult_Request(
        relyingParty: response.relyingParty,
        policy: response.policy,
        requestedAttestations: _requestedAttestationsForActiveRequest,
        sharedDataWithRelyingPartyBefore: _eventLog.includesInteractionWith(response.relyingParty),
        sessionType: DisclosureSessionType.CrossDevice,
        requestOriginBaseUrl: response.relyingParty.webUrl ?? 'https://origin.org',
        requestPurpose: [
          const LocalizedString(language: 'en', value: 'Card issuance'),
          const LocalizedString(language: 'nl', value: 'Kaart uitgifte'),
        ],
        requestType: DisclosureType.Regular,
      );
    } else {
      final requestedAttributesNotInWallet =
          _wallet.getMissingAttributeKeys(response.requestedAttributes.map((e) => e.key));
      final missingAttributes = requestedAttributesNotInWallet.map((key) {
        final associatedLabel = response.requestedAttributes.firstWhere((element) => element.key == key).label;
        return MissingAttribute(labels: associatedLabel.untranslated);
      });
      return StartDisclosureResult_RequestAttributesMissing(
        relyingParty: response.relyingParty,
        sharedDataWithRelyingPartyBefore: _eventLog.includesInteractionWith(response.relyingParty),
        sessionType: DisclosureSessionType.CrossDevice,
        requestOriginBaseUrl: response.relyingParty.webUrl ?? 'https://origin.org',
        requestPurpose: [
          const LocalizedString(language: 'en', value: 'Card issuance'),
          const LocalizedString(language: 'nl', value: 'Kaart uitgifte'),
        ],
        missingAttributes: missingAttributes.toList(),
      );
    }
  }

  Future<List<AttestationPresentation>> discloseForIssuance(String pin) async {
    assert(_activeIssuanceResponse != null, 'Can not disclose when no issuance is active');
    final result = _pinManager.checkPin(pin);
    switch (result) {
      case WalletInstructionResult_Ok():
        _itemsHaveBeenDisclosed = true;
        _eventLog.logDisclosureStep(
          _activeIssuanceResponse!.relyingParty,
          _activeIssuanceResponse!.policy,
          _requestedAttestationsForActiveRequest,
          DisclosureStatus.Success,
          purpose: [
            const LocalizedString(language: 'en', value: 'Issuance'),
            const LocalizedString(language: 'en', value: 'Uitgave'),
          ],
        );
        return _activeIssuanceResponse!.attestations.map(_assignIdIfAlreadyInWallet).toList();
      case WalletInstructionResult_InstructionError():
        throw result.error;
    }
  }

  /// Assign an id to the [AttestationPresentation]. This makes it so that the UI will
  /// register this card as 'already in wallet' and thus display it as a renewal.
  AttestationPresentation _assignIdIfAlreadyInWallet(AttestationPresentation attestation) {
    if (!_wallet.containsAttestation(attestation)) return attestation;
    return AttestationPresentation(
      identity: AttestationIdentity.fixed(id: attestation.attestationType),
      attestationType: attestation.attestationType,
      displayMetadata: attestation.displayMetadata,
      issuer: attestation.issuer,
      attributes: attestation.attributes,
    );
  }

  Future<WalletInstructionResult> acceptIssuance(String pin, Iterable<String> cardDocTypes /* empty = all */) async {
    assert(_activeIssuanceResponse != null, 'Can not accept when no issuance is active');
    final result = _pinManager.checkPin(pin);
    switch (result) {
      case WalletInstructionResult_Ok():
        final selectedCards = cardDocTypes.isEmpty
            ? _activeIssuanceResponse!.attestations
            : _activeIssuanceResponse!.attestations
                .where((card) => cardDocTypes.contains(card.attestationType))
                .toList();
        _wallet.add(selectedCards);
        selectedCards.forEach(_eventLog.logIssuance);
        _activeIssuanceResponse = null;
        _itemsHaveBeenDisclosed = false;
        return result;
      case WalletInstructionResult_InstructionError():
        throw result.error;
    }
  }

  Future<String?> cancelIssuance() async {
    if (_activeIssuanceResponse != null && !_itemsHaveBeenDisclosed /* true when already logged */) {
      _eventLog.logDisclosureStep(
        _activeIssuanceResponse!.relyingParty,
        _activeIssuanceResponse!.policy,
        _requestedAttestationsForActiveRequest,
        DisclosureStatus.Cancelled,
        purpose: [
          const LocalizedString(language: 'en', value: 'Issuance'),
          const LocalizedString(language: 'en', value: 'Uitgave'),
        ],
      );
    }
    _activeIssuanceResponse = null;
    _itemsHaveBeenDisclosed = false;
    return null;
  }
}
