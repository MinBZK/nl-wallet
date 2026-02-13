import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/configuration/flutter_app_configuration.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../configuration_repository.dart';

class ConfigurationRepositoryImpl implements ConfigurationRepository {
  final TypedWalletCore _walletCore;

  final Mapper<core.FlutterConfiguration, FlutterAppConfiguration> _flutterAppConfigurationMapper;

  ConfigurationRepositoryImpl(
    this._walletCore,
    this._flutterAppConfigurationMapper,
  );

  @override
  Stream<FlutterAppConfiguration> get observeAppConfiguration =>
      _walletCore.observeConfig().map(_flutterAppConfigurationMapper.map);
}
