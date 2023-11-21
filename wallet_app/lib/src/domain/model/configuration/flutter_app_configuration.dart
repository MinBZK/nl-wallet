import 'package:wallet_core/core.dart';

class FlutterAppConfiguration {
  final Duration idleLockTimeout;
  final Duration backgroundLockTimeout;

  const FlutterAppConfiguration({
    required this.idleLockTimeout,
    required this.backgroundLockTimeout,
  });

  factory FlutterAppConfiguration.fromFlutterConfig(FlutterConfiguration config) {
    return FlutterAppConfiguration(
      idleLockTimeout: Duration(seconds: config.inactiveLockTimeout),
      backgroundLockTimeout: Duration(seconds: config.backgroundLockTimeout),
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterAppConfiguration &&
          runtimeType == other.runtimeType &&
          idleLockTimeout == other.idleLockTimeout &&
          backgroundLockTimeout == other.backgroundLockTimeout;

  @override
  int get hashCode => idleLockTimeout.hashCode ^ backgroundLockTimeout.hashCode;

  @override
  String toString() {
    return 'AppConfiguration{idleTimeout: $idleLockTimeout, backgroundTimeout: $backgroundLockTimeout}';
  }
}
