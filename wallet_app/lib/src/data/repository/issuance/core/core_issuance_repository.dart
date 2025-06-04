import 'package:fimber/fimber.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/card/wallet_card.dart';
import '../../../../domain/model/disclosure/disclosure_session_type.dart';
import '../../../../domain/model/disclosure/disclosure_type.dart';
import '../../../../domain/model/issuance/start_issuance_result.dart';
import '../../../../domain/model/organization.dart';
import '../../../../domain/model/policy/policy.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../issuance_repository.dart';

class CoreIssuanceRepository implements IssuanceRepository {
  final TypedWalletCore _walletCore;

  final Mapper<core.AttestationPresentation, WalletCard> _attestationMapper;
  final Mapper<core.MissingAttribute, MissingAttribute> _missingAttributeMapper;
  final Mapper<core.Organization, Organization> _relyingPartyMapper;
  final Mapper<core.RequestPolicy, Policy> _requestPolicyMapper;
  final Mapper<List<core.LocalizedString>, LocalizedText> _localizedStringMapper;
  final Mapper<core.DisclosureSessionType, DisclosureSessionType> _disclosureSessionTypeMapper;
  final Mapper<core.DisclosureType, DisclosureType> _disclosureTypeMapper;

  CoreIssuanceRepository(
    this._walletCore,
    this._attestationMapper,
    this._relyingPartyMapper,
    this._missingAttributeMapper,
    this._requestPolicyMapper,
    this._localizedStringMapper,
    this._disclosureSessionTypeMapper,
    this._disclosureTypeMapper,
  );

  @override
  Future<StartIssuanceResult> startIssuance(String issuanceUri, {bool isQrCode = false}) async {
    // The first step of disclosure based issuance is identical to normal disclosure
    Fimber.d('Starting disclosure based issuance with uri: $issuanceUri. isQrCode: $isQrCode');
    final result = await _walletCore.startDisclosure(issuanceUri, isQrCode: isQrCode);
    switch (result) {
      case core.StartDisclosureResult_Request():
        final cards = _attestationMapper.mapList(result.requestedAttestations);
        final requestedAttributes = cards.asMap().map((key, value) => MapEntry(value, value.attributes));
        final relyingParty = _relyingPartyMapper.map(result.relyingParty);
        final policy = _requestPolicyMapper.map(result.policy);
        return StartIssuanceReadyToDisclose(
          relyingParty: relyingParty,
          originUrl: result.requestOriginBaseUrl,
          requestPurpose: _localizedStringMapper.map(result.requestPurpose),
          sessionType: _disclosureSessionTypeMapper.map(result.sessionType),
          type: _disclosureTypeMapper.map(result.requestType),
          requestedAttributes: requestedAttributes,
          policy: policy,
          sharedDataWithOrganizationBefore: result.sharedDataWithRelyingPartyBefore,
        );
      case core.StartDisclosureResult_RequestAttributesMissing():
        final relyingParty = _relyingPartyMapper.map(result.relyingParty);
        final missingAttributes = _missingAttributeMapper.mapList(result.missingAttributes);
        return StartIssuanceMissingAttributes(
          relyingParty: relyingParty,
          originUrl: result.requestOriginBaseUrl,
          requestPurpose: _localizedStringMapper.map(result.requestPurpose),
          sessionType: _disclosureSessionTypeMapper.map(result.sessionType),
          missingAttributes: missingAttributes,
          sharedDataWithOrganizationBefore: result.sharedDataWithRelyingPartyBefore,
        );
    }
  }

  @override
  Future<List<WalletCard>> discloseForIssuance(String pin) async {
    final result = await _walletCore.continueDisclosureBasedIssuance(pin);
    switch (result) {
      case core.DisclosureBasedIssuanceResult_Ok():
        return _attestationMapper.mapList(result.field0);
      case core.DisclosureBasedIssuanceResult_InstructionError():
        throw result.error;
    }
  }

  @override
  Future<void> acceptIssuance(String pin, Iterable<WalletCard> cards) async {
    // [cards] are currently unused, these will become relevant when we implement selective issuance.
    final result = await _walletCore.acceptIssuance(pin);
    switch (result) {
      case core.WalletInstructionResult_Ok():
        return;
      case core.WalletInstructionResult_InstructionError():
        throw result.error;
    }
  }

  @override
  Future<String?> cancelIssuance() async {
    if (await _walletCore.hasActiveDisclosureSession()) return _walletCore.cancelDisclosure();
    if (await _walletCore.hasActiveIssuanceSession()) await _walletCore.cancelIssuance();
    return null;
  }
}
