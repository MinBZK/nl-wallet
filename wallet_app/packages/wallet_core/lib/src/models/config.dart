// This file is automatically generated, so please do not edit it.
// @generated by `flutter_rust_bridge`@ 2.9.0.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import '../frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

class FlutterConfiguration {
  final int inactiveLockTimeout;
  final int backgroundLockTimeout;
  final BigInt version;

  const FlutterConfiguration({
    required this.inactiveLockTimeout,
    required this.backgroundLockTimeout,
    required this.version,
  });

  @override
  int get hashCode => inactiveLockTimeout.hashCode ^ backgroundLockTimeout.hashCode ^ version.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterConfiguration &&
          runtimeType == other.runtimeType &&
          inactiveLockTimeout == other.inactiveLockTimeout &&
          backgroundLockTimeout == other.backgroundLockTimeout &&
          version == other.version;
}
