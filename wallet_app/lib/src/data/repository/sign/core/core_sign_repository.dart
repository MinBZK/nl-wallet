import 'package:wallet_core/core.dart' as core;
import 'package:wallet_mock/mock.dart' as core show Document;
import 'package:wallet_mock/mock.dart' hide Document;

import '../../../../domain/model/card/wallet_card.dart';
import '../../../../domain/model/document.dart';
import '../../../../domain/model/organization.dart';
import '../../../../domain/model/policy/policy.dart';
import '../../../../domain/model/start_sign_result/start_sign_result.dart';
import '../../../../util/mapper/mapper.dart';
import '../sign_repository.dart';

class CoreSignRepository implements SignRepository {
  final WalletCoreForSigning _coreForSigning;

  final Mapper<core.Attestation, WalletCard> _cardMapper;
  final Mapper<core.Organization, Organization> _organizationMapper;
  final Mapper<core.RequestPolicy, Policy> _requestPolicyMapper;
  final Mapper<core.Document, Document> _documentMapper;

  CoreSignRepository(
    this._coreForSigning,
    this._cardMapper,
    this._organizationMapper,
    this._requestPolicyMapper,
    this._documentMapper,
  );

  @override
  Future<StartSignResult> startSigning(String signUri) async {
    final result = await _coreForSigning.startSigning(signUri);
    switch (result) {
      case StartSignResultReadyToDisclose():
        final cards = _cardMapper.mapList(result.requestedAttestations);
        final requestedAttributes = cards.asMap().map((key, value) => MapEntry(value, value.attributes));
        return StartSignReadyToSign(
          relyingParty: _organizationMapper.map(result.organization),
          trustProvider: _organizationMapper.map(result.trustProvider),
          policy: _requestPolicyMapper.map(result.policy),
          document: _documentMapper.map(result.document),
          requestedAttributes: requestedAttributes,
        );
      case StartSignResultRequestedAttributesMissing():
        throw UnsupportedError('We do not support this flow yet');
    }
  }

  @override
  Future<core.WalletInstructionResult> acceptAgreement(String pin) => _coreForSigning.signAgreement(pin);

  @override
  Future<void> rejectAgreement() => _coreForSigning.rejectAgreement();
}
