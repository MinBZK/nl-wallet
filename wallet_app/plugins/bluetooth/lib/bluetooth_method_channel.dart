import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

import 'bluetooth_platform_interface.dart';

/// An implementation of [BluetoothPlatform] that uses method channels.
class MethodChannelBluetooth extends BluetoothPlatform {
  /// The method channel used to interact with the native platform.
  @visibleForTesting
  final methodChannel = const MethodChannel('bluetooth');

  @override
  Future<bool> isBluetoothEnabled() async {
    final bluetoothEnabled = await methodChannel.invokeMethod<bool>('isBluetoothEnabled');
    return bluetoothEnabled == true;
  }

  @override
  Future<void> enableBluetooth() => methodChannel.invokeMethod<bool>('enableBluetooth');
}
