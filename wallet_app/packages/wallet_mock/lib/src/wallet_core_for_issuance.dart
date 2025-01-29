import 'dart:convert';

import 'package:wallet_core/core.dart';

import 'data/mock/mock_issuance_responses.dart';
import 'data/model/issuance_response.dart';
import 'log/wallet_event_log.dart';
import 'pin/pin_manager.dart';
import 'util/extension/string_extension.dart';
import 'wallet/wallet.dart';

/// Since the core doesn't support full issuance yet, this class defines a issuance
/// api which closely resembles the disclosure flow. Once [WalletCore] does support
/// issuance the the mock can be implemented there (through [WalletCoreMock]) and this
/// class should be deleted.
class WalletCoreForIssuance {
  final PinManager _pinManager;
  final Wallet _wallet;
  final WalletEventLog _eventLog;

  IssuanceResponse? _activeIssuanceResponse;
  bool _itemsHaveBeenDisclosed = false;

  /// Get the cards/attributes that have to be disclosed to fulfill [_activeIssuanceResponse], assumes [_activeIssuanceResponse] is non null.
  List<DisclosureCard> get _disclosureCardsForActiveRequest => _wallet.getDisclosureCards(
        _activeIssuanceResponse!.requestedAttributes.map(
          (attribute) => attribute.key,
        ),
      );

  WalletCoreForIssuance(this._pinManager, this._wallet, this._eventLog);

  Future<StartIssuanceResult> startIssuance(String uri) async {
    // Look up the associated response
    final jsonPayload = jsonDecode(Uri.decodeComponent(Uri.parse(uri).fragment));
    final issuanceId = jsonPayload['id'] as String;
    final response = _activeIssuanceResponse = kIssuanceResponses.firstWhere((element) => element.id == issuanceId);

    final issuancePossible = _wallet.containsAttributes(response.requestedAttributes.map((e) => e.key));
    if (issuancePossible) {
      return StartIssuanceResultReadyToDisclose(
        response.organization,
        response.policy,
        _disclosureCardsForActiveRequest,
      );
    } else {
      final requestedAttributesNotInWallet =
          _wallet.getMissingAttributeKeys(response.requestedAttributes.map((e) => e.key));
      final missingAttributes = requestedAttributesNotInWallet.map((key) {
        final associatedLabel = response.requestedAttributes.firstWhere((element) => element.key == key).label;
        return MissingAttribute(labels: associatedLabel.untranslated);
      });
      return StartIssuanceResultRequestedAttributesMissing(
        response.organization,
        response.policy,
        missingAttributes.toList(),
      );
    }
  }

  Future<WalletInstructionResult> discloseForIssuance(String pin) async {
    assert(_activeIssuanceResponse != null, 'Can not disclose when no issuance is active');
    final result = _pinManager.checkPin(pin);
    if (result is WalletInstructionResult_Ok) {
      _itemsHaveBeenDisclosed = true;
      _eventLog.logDisclosureStep(
        _activeIssuanceResponse!.organization,
        _activeIssuanceResponse!.policy,
        _disclosureCardsForActiveRequest,
        DisclosureStatus.Success,
        purpose: [
          LocalizedString(language: 'en', value: 'Issuance'),
          LocalizedString(language: 'en', value: 'Uitgave'),
        ],
      );
    }

    return result;
  }

  Future<List<Attestation>> proceedIssuance() async {
    assert(_activeIssuanceResponse != null, 'Can not issue when no issuance is active');
    return _activeIssuanceResponse!.attestations;
  }

  Future<void> acceptIssuance(List<String> cardDocTypes) async {
    assert(_activeIssuanceResponse != null, 'Can not accept when no issuance is active');
    final selectedCards =
        _activeIssuanceResponse!.attestations.where((card) => cardDocTypes.contains(card.attestationType)).toList();
    _wallet.add(selectedCards);
    selectedCards.forEach(_eventLog.logIssuance);
    _activeIssuanceResponse = null;
    _itemsHaveBeenDisclosed = false;
  }

  Future<void> cancelIssuance() async {
    if (_activeIssuanceResponse != null && !_itemsHaveBeenDisclosed /* true when already logged */) {
      _eventLog.logDisclosureStep(
        _activeIssuanceResponse!.organization,
        _activeIssuanceResponse!.policy,
        _disclosureCardsForActiveRequest,
        DisclosureStatus.Cancelled,
        purpose: [
          LocalizedString(language: 'en', value: 'Issuance'),
          LocalizedString(language: 'en', value: 'Uitgave'),
        ],
      );
    }
    _activeIssuanceResponse = null;
    _itemsHaveBeenDisclosed = false;
  }

  Future<Organization> getIssuer(String docType) async {
    final relatedIssuanceResponse = kIssuanceResponses
        .firstWhere((response) => response.attestations.any((card) => card.attestationType == docType));
    return relatedIssuanceResponse.organization;
  }
}

sealed class StartIssuanceResult {
  final Organization organization;
  final RequestPolicy policy;

  StartIssuanceResult(this.organization, this.policy);
}

class StartIssuanceResultReadyToDisclose extends StartIssuanceResult {
  final List<DisclosureCard> disclosureCards;

  StartIssuanceResultReadyToDisclose(super.organization, super.policy, this.disclosureCards);
}

class StartIssuanceResultRequestedAttributesMissing extends StartIssuanceResult {
  final List<MissingAttribute> missingAttributes;

  StartIssuanceResultRequestedAttributesMissing(super.organization, super.policy, this.missingAttributes);
}
