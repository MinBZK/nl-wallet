import '../../../../domain/model/configuration/app_configuration.dart';
import '../configuration_repository.dart';

class MockConfigurationRepository implements ConfigurationRepository {
  @override
  Stream<AppConfiguration> get appConfiguration => Stream.value(
        const AppConfiguration(
          backgoundLockTimeout: Duration(minutes: 5),
          idleLockTimeout: Duration(minutes: 20),
        ),
      );
}
