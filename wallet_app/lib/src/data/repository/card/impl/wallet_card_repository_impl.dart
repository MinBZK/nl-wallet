import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/card/wallet_card.dart';
import '../../../../domain/model/configuration/flutter_app_configuration.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../util/mixin/pid_filter_mixin.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../../../source/wallet_datasource.dart';
import '../wallet_card_repository.dart';

class WalletCardRepositoryImpl extends WalletCardRepository with PidFilterMixin {
  final WalletDataSource _dataSource;
  final TypedWalletCore _walletCore;
  final Mapper<core.FlutterConfiguration, FlutterAppConfiguration> _flutterAppConfigurationMapper;

  WalletCardRepositoryImpl(this._dataSource, this._walletCore, this._flutterAppConfigurationMapper);

  @override
  AppConfigurationProvider get configProvider =>
      () async => _flutterAppConfigurationMapper.map(await _walletCore.observeConfig().first);

  @override
  Stream<List<WalletCard>> observeWalletCards({bool filterDuplicatePids = true}) {
    if (!filterDuplicatePids) return _dataSource.observeCards();
    return _dataSource.observeCards().asyncMap(filterDuplicatePidCards);
  }

  @override
  Future<bool> exists(String attestationId) async => await _dataSource.read(attestationId) != null;

  @override
  Future<List<WalletCard>> readAll({bool filterDuplicatePids = true}) async {
    final cards = await _dataSource.readAll();
    if (!filterDuplicatePids) return cards;
    return filterDuplicatePidCards(cards);
  }

  @override
  Future<WalletCard> read(String attestationId) async => (await _dataSource.read(attestationId))!;

  @override
  Future<void> delete(String pin, String attestationId) async {
    final result = await _dataSource.delete(pin, attestationId);
    switch (result) {
      case core.WalletInstructionResult_Ok():
        return;
      case core.WalletInstructionResult_InstructionError():
        throw result.error;
    }
  }
}
