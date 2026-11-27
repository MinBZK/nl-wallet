import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

import 'bluetooth_platform_interface.dart';

/// An implementation of [BluetoothPlatform] that uses method channels.
class MethodChannelBluetooth extends BluetoothPlatform {
  /// The method channel used to interact with the native platform.
  @visibleForTesting
  final methodChannel = const MethodChannel('bluetooth');

  @visibleForTesting
  final stateChannel = const EventChannel('bluetooth/state');

  // An EventChannel supports one listener per channel: a second receiveBroadcastStream() takes over the platform
  // stream, and cancelling the first one then ends it for both.
  late final Stream<bool> _enabledStream = stateChannel.receiveBroadcastStream().map((enabled) => enabled == true);

  @override
  Future<bool> isBluetoothEnabled() async {
    final bluetoothEnabled = await methodChannel.invokeMethod<bool>('isBluetoothEnabled');
    return bluetoothEnabled == true;
  }

  @override
  Stream<bool> observeBluetoothEnabled() => _enabledStream;

  @override
  Future<void> enableBluetooth() => methodChannel.invokeMethod<bool>('enableBluetooth');
}
