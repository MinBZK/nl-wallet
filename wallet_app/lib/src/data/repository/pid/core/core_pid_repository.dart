import 'dart:async';

import 'package:collection/collection.dart';
import 'package:wallet_core/core.dart' as core;
import 'package:wallet_core/core.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/card/wallet_card.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../pid_repository.dart';

class CorePidRepository extends PidRepository {
  final TypedWalletCore _walletCore;
  final Mapper<AttestationPresentation, WalletCard> _cardMapper;

  CorePidRepository(
    this._walletCore,
    this._cardMapper,
  );

  @override
  Future<String> getPidIssuanceUrl() => _walletCore.createPidIssuanceRedirectUri();

  @override
  Future<String> getPidRenewalUrl() => _walletCore.createPidRenewalRedirectUri();

  @override
  Future<List<DataAttribute>> continuePidIssuance(String uri) async {
    final result = await _walletCore.continuePidIssuance(uri);
    return result.map(_cardMapper.map).map((attestation) => attestation.attributes).flattened.toList();
  }

  @override
  Future<void> cancelIssuance() => _walletCore.cancelIssuance();

  @override
  Future<bool> hasActiveIssuanceSession() async => await _walletCore.getWalletState() is core.WalletEvent_Issuance;

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
