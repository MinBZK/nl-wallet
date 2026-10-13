import 'dart:async';

import 'package:collection/collection.dart';
import 'package:wallet_core/core.dart' as core;
import 'package:wallet_core/core.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/card/wallet_card.dart';
import '../../../../domain/model/configuration/flutter_app_configuration.dart';
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
  Future<List<DataAttribute>> continuePidIssuance(String uri) async {
    final result = await _walletCore.continueIssuance(uri);
    final attestations = switch (result) {
      core.IssuanceContinueResult_Previews(field0: final attestations) => attestations,
      // The core rejects PID issuance without previews, so this should never happen.
      core.IssuanceContinueResult_NoPreviews() => throw StateError('PID issuance session contains no previews'),
    };
    final cards = attestations.map(_cardMapper.map).toList();
    final filteredCards = await filterDuplicatePidCards(cards);
    return filteredCards.map((attestation) => attestation.attributes).flattened.toList();
  }

  @override
  Future<TransferState> acceptIssuance(String pin) async {
    final result = await _walletCore.acceptPidIssuance(pin);
    switch (result) {
      case PidIssuanceResult_Ok():
        return result.transferAvailable ? TransferState.available : TransferState.unavailable;
      case PidIssuanceResult_InstructionError():
        throw result.error; // Makes sure we expose the [WalletInstructionError] as an error.
    }
  }
}
