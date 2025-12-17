import 'package:equatable/equatable.dart';
import 'package:flutter/foundation.dart';
import 'package:wallet_core/core.dart';

@immutable
class FlutterAppConfiguration extends Equatable {
  final Duration idleLockTimeout;
  final Duration idleWarningTimeout;
  final Duration backgroundLockTimeout;
  final String staticAssetsBaseUrl;
  final List<String> pidAttestationTypes;
  final String version;
  final String environment;

  const FlutterAppConfiguration({
    required this.idleLockTimeout,
    required this.idleWarningTimeout,
    required this.backgroundLockTimeout,
    required this.staticAssetsBaseUrl,
    required this.pidAttestationTypes,
    required this.version,
    required this.environment,
  });

  factory FlutterAppConfiguration.fromFlutterConfig(FlutterConfiguration config) {
    return FlutterAppConfiguration(
      idleLockTimeout: Duration(seconds: config.inactiveLockTimeout),
      idleWarningTimeout: Duration(seconds: config.inactiveWarningTimeout),
      backgroundLockTimeout: Duration(seconds: config.backgroundLockTimeout),
      staticAssetsBaseUrl: config.staticAssetsBaseUrl,
      pidAttestationTypes: config.pidAttestationTypes,
      version: config.version,
      environment: config.environment,
    );
  }

  String get versionAndEnvironment => '$version ($environment)';

  @override
  List<Object?> get props => [
    idleLockTimeout,
    idleWarningTimeout,
    backgroundLockTimeout,
    staticAssetsBaseUrl,
    pidAttestationTypes,
    version,
  ];
}
