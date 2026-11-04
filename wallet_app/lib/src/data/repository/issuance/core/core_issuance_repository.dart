import 'package:fimber/fimber.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/card/wallet_card.dart';
import '../../../../domain/model/disclosure/disclose_card_request.dart';
import '../../../../domain/model/disclosure/disclosure_session_type.dart';
import '../../../../domain/model/issuance/continue_issuance_result.dart';
import '../../../../domain/model/issuance/start_issuance_result.dart';
import '../../../../domain/model/organization.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../issuance_repository.dart';

class CoreIssuanceRepository implements IssuanceRepository {
  final TypedWalletCore _walletCore;

  final Mapper<core.AttestationPresentation, WalletCard> _attestationMapper;
  final Mapper<core.MissingAttribute, MissingAttribute> _missingAttributeMapper;
  final Mapper<core.Organization, Organization> _relyingPartyMapper;
  final Mapper<List<core.LocalizedString>, LocalizedText> _localizedStringMapper;
  final Mapper<core.DisclosureSessionType, DisclosureSessionType> _disclosureSessionTypeMapper;

  CoreIssuanceRepository(
    this._walletCore,
    this._attestationMapper,
    this._relyingPartyMapper,
    this._missingAttributeMapper,
    this._localizedStringMapper,
    this._disclosureSessionTypeMapper,
  );

  @override
  Future<StartIssuanceResult> startIssuance(String issuanceUri, {bool isQrCode = false}) async {
    // The first step of disclosure based issuance is identical to normal disclosure
    Fimber.d('Starting disclosure based issuance with uri: $issuanceUri. isQrCode: $isQrCode');
    final result = await _walletCore.startDisclosure(issuanceUri, isQrCode: isQrCode);
    switch (result) {
      case core.StartDisclosureResult_Request():
        final List<DiscloseCardRequest> requests = result.disclosureOptions
            .map((it) => DiscloseCardRequest(candidates: _attestationMapper.mapList(it.field0)))
            .toList();
        final relyingParty = _relyingPartyMapper.map(result.relyingParty);
        return StartIssuanceReadyToDisclose(
          relyingParty: relyingParty,
          requestPurpose: _localizedStringMapper.map(result.requestPurpose),
          sessionType: _disclosureSessionTypeMapper.map(result.sessionType),
          cardRequests: requests,
        );
      case core.StartDisclosureResult_RequestAttributesMissing():
        final relyingParty = _relyingPartyMapper.map(result.relyingParty);
        final missingAttributes = _missingAttributeMapper.mapList(result.missingAttributes);
        return StartIssuanceMissingAttributes(
          relyingParty: relyingParty,
          sessionType: _disclosureSessionTypeMapper.map(result.sessionType),
          missingAttributes: missingAttributes,
        );
    }
  }

  @override
  Future<StartIssuanceResult> startIssuanceFromOffer(String issuanceUri, {bool isQrCode = false}) async {
    final result = await _walletCore.startIssuanceFromOffer(issuanceUri);
    switch (result) {
      case core.IssuanceStartResult_AuthorizationUrl(field0: final authUrl):
        return StartIssuanceAuthorizationRequired(authUrl);
      case core.IssuanceStartResult_Previews(field0: final cards):
        return StartIssuancePreAuthorizedOffer(_attestationMapper.mapList(cards));
      case core.IssuanceStartResult_NoPreviews(:final issuer):
        return StartIssuanceNoPreviews(issuer: _relyingPartyMapper.map(issuer));
    }
  }

  @override
  Future<List<WalletCard>> discloseForIssuance(String pin, List<int> selectedIndices) async {
    final result = await _walletCore.continueDisclosureBasedIssuance(pin, selectedIndices);
    switch (result) {
      case core.DisclosureBasedIssuanceInstructionResult_Ok(field0: final result):
        switch (result) {
          case core.DisclosureBasedIssuanceResult_Previews(field0: final cards):
            return _attestationMapper.mapList(cards);
          case core.DisclosureBasedIssuanceResult_NoIssuableAttestations():
            return [];
          case core.DisclosureBasedIssuanceResult_NoPreviews():
            throw UnsupportedError('Disclosure based issuance without previews is not supported yet');
        }
      case core.DisclosureBasedIssuanceInstructionResult_InstructionError():
        throw result.error;
    }
  }

  @override
  Future<ContinueIssuanceResult> continueIssuance(String uri) async {
    final result = await _walletCore.continueIssuance(uri);
    switch (result) {
      case core.IssuanceContinueResult_Previews(field0: final cards):
        return ContinueIssuancePreviews(_attestationMapper.mapList(cards));
      case core.IssuanceContinueResult_NoPreviews(:final issuer):
        return ContinueIssuanceNoPreviews(issuer: _relyingPartyMapper.map(issuer));
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
}
