import '../../../../domain/model/configuration/app_configuration.dart';
import '../../../../wallet_core/typed_wallet_core.dart';
import '../configuration_repository.dart';

class CoreConfigurationRepository implements ConfigurationRepository {
  final TypedWalletCore _walletCore;

  CoreConfigurationRepository(this._walletCore);

  @override
  Stream<AppConfiguration> get appConfiguration =>
      _walletCore.observeConfig().map((event) => AppConfiguration.fromFlutterConfig(event));
}
