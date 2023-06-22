import '../../../domain/model/configuration/app_configuration.dart';

abstract class ConfigurationRepository {
  Stream<AppConfiguration> get appConfiguration;
}
