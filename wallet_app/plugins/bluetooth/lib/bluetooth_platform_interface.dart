import 'package:plugin_platform_interface/plugin_platform_interface.dart';

import 'bluetooth_method_channel.dart';

abstract class BluetoothPlatform extends PlatformInterface {
  /// Constructs a BluetoothPlatform.
  BluetoothPlatform() : super(token: _token);

  static final Object _token = Object();

  static BluetoothPlatform _instance = MethodChannelBluetooth();

  /// The default instance of [BluetoothPlatform] to use.
  ///
  /// Defaults to [MethodChannelBluetooth].
  static BluetoothPlatform get instance => _instance;

  /// Platform-specific implementations should set this with their own
  /// platform-specific class that extends [BluetoothPlatform] when
  /// they register themselves.
  static set instance(BluetoothPlatform instance) {
    PlatformInterface.verifyToken(instance, _token);
    _instance = instance;
  }

  Future<bool> isBluetoothEnabled() {
    throw UnimplementedError('isBluetoothEnabled() has not been implemented.');
  }

  Future<void> enableBluetooth() {
    throw UnimplementedError('enableBluetooth() has not been implemented.');
  }
}
