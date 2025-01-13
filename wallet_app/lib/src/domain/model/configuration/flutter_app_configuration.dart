import 'package:flutter/foundation.dart';
import 'package:wallet_core/core.dart';

@immutable
class FlutterAppConfiguration {
  final Duration idleLockTimeout;
  final Duration backgroundLockTimeout;
  final int version;

  const FlutterAppConfiguration({
    required this.idleLockTimeout,
    required this.backgroundLockTimeout,
    required this.version,
  });

  factory FlutterAppConfiguration.fromFlutterConfig(FlutterConfiguration config) {
    return FlutterAppConfiguration(
      idleLockTimeout: Duration(seconds: config.inactiveLockTimeout),
      backgroundLockTimeout: Duration(seconds: config.backgroundLockTimeout),
      version: config.version,
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterAppConfiguration &&
          runtimeType == other.runtimeType &&
          idleLockTimeout == other.idleLockTimeout &&
          backgroundLockTimeout == other.backgroundLockTimeout &&
          version == other.version;

  @override
  int get hashCode => idleLockTimeout.hashCode ^ backgroundLockTimeout.hashCode ^ version.hashCode;

  @override
  String toString() {
    return 'AppConfiguration{idleTimeout: $idleLockTimeout, backgroundTimeout: $backgroundLockTimeout, version: $version}';
  }
}
