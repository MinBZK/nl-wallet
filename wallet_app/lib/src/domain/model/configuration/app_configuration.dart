import '../../../../bridge_generated.dart';

class AppConfiguration {
  final Duration idleLockTimeout;
  final Duration backgoundLockTimeout;

  const AppConfiguration({
    required this.idleLockTimeout,
    required this.backgoundLockTimeout,
  });

  factory AppConfiguration.fromFlutterConfig(FlutterConfiguration config) {
    return AppConfiguration(
      idleLockTimeout: Duration(seconds: config.inactiveLockTimeout),
      backgoundLockTimeout: Duration(seconds: config.backgroundLockTimeout),
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is AppConfiguration &&
          runtimeType == other.runtimeType &&
          idleLockTimeout == other.idleLockTimeout &&
          backgoundLockTimeout == other.backgoundLockTimeout;

  @override
  int get hashCode => idleLockTimeout.hashCode ^ backgoundLockTimeout.hashCode;

  @override
  String toString() {
    return 'AppConfiguration{idleTimeout: $idleLockTimeout, backgroundTimeout: $backgoundLockTimeout}';
  }
}
