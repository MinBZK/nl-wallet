import '../../../domain/model/configuration/flutter_app_configuration.dart';

abstract class ConfigurationRepository {
  Stream<FlutterAppConfiguration> get observeAppConfiguration;

  // When 'true' the config is expired (and thus the key material/trust anchors can not be trusted).
  // All app interactions should be blocked until it flips back to 'false', indicating a valid config is available.
  Stream<bool> get observeConfigExpired;
}
