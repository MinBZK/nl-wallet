import 'dart:async';

import 'package:collection/collection.dart';
import 'package:wallet_core/core.dart' as core;
import 'package:wallet_core/core.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/card/wallet_card.dart';
import '../../../../domain/model/configuration/flutter_app_configuration.dart';
import '../../../../domain/model/issuance/continue_pid_issuance_result.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../util/mixin/pid_filter_mixin.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../pid_repository.dart';

class CorePidRepository extends PidRepository with PidFilterMixin {
  final TypedWalletCore _walletCore;
  final Mapper<AttestationPresentation, WalletCard> _cardMapper;
  final Mapper<core.FlutterConfiguration, FlutterAppConfiguration> _flutterAppConfigurationMapper;

  CorePidRepository(
    this._walletCore,
    this._cardMapper,
    this._flutterAppConfigurationMapper,
  );

  @override
  AppConfigurationProvider get configProvider =>
      () async => _flutterAppConfigurationMapper.map(await _walletCore.observeConfig().first);

  @override
  Future<String> getPidIssuanceUrl() => _walletCore.createPidIssuanceRedirectUri();

  @override
  Future<String> getPidRenewalUrl() => _walletCore.createPidRenewalRedirectUri();

  @override
  Future<ContinuePidIssuanceResult> continuePidIssuance(String uri) async {
    final result = await _walletCore.continueIssuance(uri);
    switch (result) {
      case core.IssuanceContinueResult_Previews(field0: final attestations):
        return ContinuePidIssuanceResult.previews(await _mapToAttributes(attestations));
      case core.IssuanceContinueResult_NoPreviews():
        return const ContinuePidIssuanceResult.noPreviews();
    }
  }

  @override
  Future<TransferState> acceptIssuance(String pin) async => _mapTransferState(await _walletCore.acceptPidIssuance(pin));

  @override
  Future<List<DataAttribute>> issuePidForReview(String pin) async {
    final result = await _walletCore.issueCredentialsForReview(pin);
    switch (result) {
      case core.IssueCredentialsResult_Ok(field0: final attestations):
        return _mapToAttributes(attestations);
      case core.IssueCredentialsResult_InstructionError():
        throw result.error; // Makes sure we expose the [WalletInstructionError] as an error.
    }
  }

  @override
  Future<TransferState> commitIssuedPid(String pin) async => _mapTransferState(await _walletCore.commitIssuedPid(pin));

  Future<List<DataAttribute>> _mapToAttributes(List<AttestationPresentation> attestations) async {
    final cards = attestations.map(_cardMapper.map).toList();
    final filteredCards = await filterDuplicatePidCards(cards);
    return filteredCards.map((attestation) => attestation.attributes).flattened.toList();
  }

  TransferState _mapTransferState(PidIssuanceResult result) {
    switch (result) {
      case PidIssuanceResult_Ok():
        return result.transferAvailable ? TransferState.available : TransferState.unavailable;
      case PidIssuanceResult_InstructionError():
        throw result.error; // Makes sure we expose the [WalletInstructionError] as an error.
    }
  }
}
