import '../../../../domain/model/configuration/flutter_app_configuration.dart';
import '../configuration_repository.dart';

class MockConfigurationRepository implements ConfigurationRepository {
  @override
  Stream<FlutterAppConfiguration> get appConfiguration => Stream.value(
        const FlutterAppConfiguration(
          backgroundLockTimeout: Duration(minutes: 5),
          idleLockTimeout: Duration(minutes: 20),
        ),
      );
}
