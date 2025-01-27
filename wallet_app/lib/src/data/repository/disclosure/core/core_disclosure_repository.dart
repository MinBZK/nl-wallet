import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/disclosure/disclosure_session_type.dart';
import '../../../../domain/model/disclosure/disclosure_type.dart';
import '../../../../domain/model/organization.dart';
import '../../../../domain/model/policy/policy.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../disclosure_repository.dart';

class CoreDisclosureRepository implements DisclosureRepository {
  final TypedWalletCore _walletCore;

  final Mapper<core.DisclosureCard, WalletCard> _disclosureCardMapper;
  final Mapper<core.MissingAttribute, MissingAttribute> _missingAttributeMapper;
  final Mapper<core.Organization, Organization> _relyingPartyMapper;
  final Mapper<core.RequestPolicy, Policy> _requestPolicyMapper;
  final Mapper<List<core.LocalizedString>, LocalizedText> _localizedStringMapper;
  final Mapper<core.DisclosureSessionType, DisclosureSessionType> _disclosureSessionTypeMapper;
  final Mapper<core.DisclosureType, DisclosureType> _disclosureTypeMapper;

  CoreDisclosureRepository(
    this._walletCore,
    this._disclosureCardMapper,
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
    return result.map(
      request: (value) {
        final cards = _disclosureCardMapper.mapList(value.requestedCards);
        final requestedAttributes = cards.asMap().map((key, value) => MapEntry(value, value.attributes));
        final relyingParty = _relyingPartyMapper.map(value.relyingParty);
        final policy = _requestPolicyMapper.map(value.policy);
        return StartDisclosureReadyToDisclose(
          relyingParty,
          value.requestOriginBaseUrl,
          _localizedStringMapper.map(value.requestPurpose),
          _disclosureSessionTypeMapper.map(value.sessionType),
          _disclosureTypeMapper.map(value.requestType),
          requestedAttributes,
          policy,
          sharedDataWithOrganizationBefore: value.sharedDataWithRelyingPartyBefore,
        );
      },
      requestAttributesMissing: (value) {
        final relyingParty = _relyingPartyMapper.map(value.relyingParty);
        final missingAttributes = _missingAttributeMapper.mapList(value.missingAttributes);
        return StartDisclosureMissingAttributes(
          relyingParty,
          value.requestOriginBaseUrl,
          _localizedStringMapper.map(value.requestPurpose),
          _disclosureSessionTypeMapper.map(value.sessionType),
          missingAttributes,
          sharedDataWithOrganizationBefore: value.sharedDataWithRelyingPartyBefore,
        );
      },
    );
  }

  @override
  Future<String?> cancelDisclosure() => _walletCore.cancelDisclosure();

  @override
  Future<bool> hasActiveDisclosureSession() => _walletCore.hasActiveDisclosureSession();

  @override
  Future<core.AcceptDisclosureResult> acceptDisclosure(String pin) => _walletCore.acceptDisclosure(pin);
}
