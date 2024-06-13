import '../../../../domain/model/configuration/flutter_app_configuration.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../configuration_repository.dart';

class ConfigurationRepositoryImpl implements ConfigurationRepository {
  final TypedWalletCore _walletCore;

  ConfigurationRepositoryImpl(this._walletCore);

  @override
  Stream<FlutterAppConfiguration> get appConfiguration =>
      _walletCore.observeConfig().map(FlutterAppConfiguration.fromFlutterConfig);
}
