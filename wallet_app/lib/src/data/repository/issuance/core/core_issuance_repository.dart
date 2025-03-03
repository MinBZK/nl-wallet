import 'package:wallet_core/core.dart' as core;
import 'package:wallet_mock/mock.dart' hide StartIssuanceResult;

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/card/wallet_card.dart';
import '../../../../domain/model/issuance/continue_issuance_result.dart';
import '../../../../domain/model/issuance/start_issuance_result.dart';
import '../../../../domain/model/organization.dart';
import '../../../../domain/model/policy/policy.dart';
import '../../../../util/mapper/mapper.dart';
import '../issuance_repository.dart';

class CoreIssuanceRepository implements IssuanceRepository {
  /// Replace with [WalletCore] once it supports issuance.
  final WalletCoreForIssuance _core;

  final Mapper<core.Attestation, WalletCard> _cardMapper;
  final Mapper<core.DisclosureCard, WalletCard> _disclosureCardMapper;
  final Mapper<core.MissingAttribute, MissingAttribute> _missingAttributeMapper;
  final Mapper<core.Organization, Organization> _organizationMapper;
  final Mapper<core.RequestPolicy, Policy> _requestPolicyMapper;

  CoreIssuanceRepository(
    this._core,
    this._cardMapper,
    this._disclosureCardMapper,
    this._organizationMapper,
    this._missingAttributeMapper,
    this._requestPolicyMapper,
  );

  @override
  Future<StartIssuanceResult> startIssuance(String disclosureUri) async {
    final result = await _core.startIssuance(disclosureUri);
    switch (result) {
      case StartIssuanceResultReadyToDisclose():
        final cards = _disclosureCardMapper.mapList(result.disclosureCards);
        final requestedAttributes = cards.asMap().map((key, value) => MapEntry(value, value.attributes));
        return StartIssuanceReadyToDisclose(
          relyingParty: _organizationMapper.map(result.organization),
          policy: _requestPolicyMapper.map(result.policy),
          requestedAttributes: requestedAttributes,
        );
      case StartIssuanceResultRequestedAttributesMissing():
        return StartIssuanceMissingAttributes(
          relyingParty: _organizationMapper.map(result.organization),
          policy: _requestPolicyMapper.map(result.policy),
          missingAttributes: _missingAttributeMapper.mapList(result.missingAttributes),
        );
    }
  }

  @override
  Future<core.WalletInstructionResult> discloseForIssuance(String pin) => _core.discloseForIssuance(pin);

  @override
  Future<void> acceptIssuance(Iterable<String> cardDocTypes) => _core.acceptIssuance(cardDocTypes.toList());

  @override
  Future<void> cancelIssuance() => _core.cancelIssuance();

  @override
  Future<ContinueIssuanceResult> proceedIssuance() async {
    final cards = await _core.proceedIssuance();
    return ContinueIssuanceResult(_cardMapper.mapList(cards));
  }
}
