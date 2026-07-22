import 'package:wallet_core/core.dart' as core;
import 'package:wallet_mock/mock.dart' as core show Document;
import 'package:wallet_mock/mock.dart' hide Document;

import '../../../../domain/model/card/wallet_card.dart';
import '../../../../domain/model/document.dart';
import '../../../../domain/model/organization.dart';
import '../../../../domain/model/policy/policy.dart';
import '../../../../domain/model/start_sign_result/start_sign_result.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../util/sentry/sentry_breadcrumbs.dart';
import '../sign_repository.dart';

class CoreSignRepository implements SignRepository {
  final WalletCoreForSigning _coreForSigning;

  final Mapper<core.AttestationPresentation, WalletCard> _cardMapper;
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
  Future<StartSignResult> startSigning(String signUri) => _callWithFlowBreadcrumb(
    'sign.start',
    failureCode: 'sign.fail.start',
    runnable: () => _startSigning(signUri),
  );

  Future<StartSignResult> _startSigning(String signUri) async {
    final result = await _coreForSigning.startSigning(signUri);
    switch (result) {
      case StartSignResultReadyToDisclose():
        return StartSignReadyToSign(
          relyingParty: _organizationMapper.map(result.organization),
          trustProvider: _organizationMapper.map(result.trustProvider),
          policy: _requestPolicyMapper.map(result.policy),
          document: _documentMapper.map(result.document),
          requestedCards: _cardMapper.mapList(result.requestedAttestations),
        );
      case StartSignResultRequestedAttributesMissing():
        throw UnsupportedError('We do not support this flow yet');
    }
  }

  @override
  Future<core.WalletInstructionResult> acceptAgreement(String pin) => _callWithFlowBreadcrumb(
    'sign.accept',
    failureCode: 'sign.fail.accept',
    runnable: () => _coreForSigning.signAgreement(pin),
  );

  @override
  Future<void> rejectAgreement() => _callWithFlowBreadcrumb(
    'sign.reject',
    failureCode: 'sign.fail.reject',
    runnable: _coreForSigning.rejectAgreement,
  );

  Future<T> _callWithFlowBreadcrumb<T>(
    String code, {
    required String failureCode,
    required Future<T> Function() runnable,
  }) async {
    await SentryBreadcrumbs.flow(code);
    try {
      return await runnable();
    } catch (_) {
      await SentryBreadcrumbs.flow(failureCode);
      rethrow;
    }
  }
}
