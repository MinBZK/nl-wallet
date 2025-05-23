import 'package:equatable/equatable.dart';
import 'package:flutter/foundation.dart';
import 'package:wallet_core/core.dart';

@immutable
class FlutterAppConfiguration extends Equatable {
  final Duration idleLockTimeout;
  final Duration idleWarningTimeout;
  final Duration backgroundLockTimeout;
  final String staticAssetsBaseUrl;
  final int version;

  const FlutterAppConfiguration({
    required this.idleLockTimeout,
    required this.idleWarningTimeout,
    required this.backgroundLockTimeout,
    required this.staticAssetsBaseUrl,
    required this.version,
  });

  factory FlutterAppConfiguration.fromFlutterConfig(FlutterConfiguration config) {
    return FlutterAppConfiguration(
      idleLockTimeout: Duration(seconds: config.inactiveLockTimeout),
      idleWarningTimeout: Duration(seconds: config.inactiveWarningTimeout),
      backgroundLockTimeout: Duration(seconds: config.backgroundLockTimeout),
      staticAssetsBaseUrl: config.staticAssetsBaseUrl,
      version: config.version.toInt(),
    );
  }

  @override
  List<Object?> get props => [
        idleLockTimeout,
        idleWarningTimeout,
        backgroundLockTimeout,
        staticAssetsBaseUrl,
        version,
      ];
}
