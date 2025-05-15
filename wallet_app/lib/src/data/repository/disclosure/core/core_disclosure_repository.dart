import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/card/wallet_card.dart';
import '../../../../domain/model/disclosure/disclosure_session_type.dart';
import '../../../../domain/model/disclosure/disclosure_type.dart';
import '../../../../domain/model/organization.dart';
import '../../../../domain/model/policy/policy.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../disclosure_repository.dart';

class CoreDisclosureRepository implements DisclosureRepository {
  final TypedWalletCore _walletCore;

  final Mapper<core.Attestation, WalletCard> _attestationMapper;
  final Mapper<core.MissingAttribute, MissingAttribute> _missingAttributeMapper;
  final Mapper<core.Organization, Organization> _relyingPartyMapper;
  final Mapper<core.RequestPolicy, Policy> _requestPolicyMapper;
  final Mapper<List<core.LocalizedString>, LocalizedText> _localizedStringMapper;
  final Mapper<core.DisclosureSessionType, DisclosureSessionType> _disclosureSessionTypeMapper;
  final Mapper<core.DisclosureType, DisclosureType> _disclosureTypeMapper;

  CoreDisclosureRepository(
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
  Future<StartDisclosureResult> startDisclosure(String disclosureUri, {bool isQrCode = false}) async {
    final result = await _walletCore.startDisclosure(disclosureUri, isQrCode: isQrCode);
    switch (result) {
      case core.StartDisclosureResult_Request():
        final cards = _attestationMapper.mapList(result.requestedAttestations);
        final requestedAttributes = cards.asMap().map((key, value) => MapEntry(value, value.attributes));
        final relyingParty = _relyingPartyMapper.map(result.relyingParty);
        final policy = _requestPolicyMapper.map(result.policy);
        return StartDisclosureReadyToDisclose(
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
        return StartDisclosureMissingAttributes(
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
  Future<String?> cancelDisclosure() => _walletCore.cancelDisclosure();

  @override
  Future<bool> hasActiveDisclosureSession() => _walletCore.hasActiveDisclosureSession();

  @override
  Future<String?> acceptDisclosure(String pin) async {
    final result = await _walletCore.acceptDisclosure(pin);
    switch (result) {
      case core.AcceptDisclosureResult_Ok():
        return result.returnUrl;
      case core.AcceptDisclosureResult_InstructionError():
        throw result.error;
    }
  }
}
