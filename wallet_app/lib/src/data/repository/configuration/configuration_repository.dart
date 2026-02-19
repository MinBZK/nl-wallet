import '../../../domain/model/configuration/flutter_app_configuration.dart';

abstract class ConfigurationRepository {
  Stream<FlutterAppConfiguration> get observeAppConfiguration;
}
