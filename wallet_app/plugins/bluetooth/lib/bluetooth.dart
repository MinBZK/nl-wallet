import 'bluetooth_platform_interface.dart';

/// Provides methods to interact with the device's Bluetooth hardware.
class Bluetooth {
  /// Checks whether Bluetooth is currently enabled on the host device.
  ///
  /// Returns a [Future] that resolves to `true` if Bluetooth is enabled,
  /// and `false` otherwise.
  Future<bool> isEnabled() => BluetoothPlatform.instance.isBluetoothEnabled();

  /// Emits whether Bluetooth is enabled, whenever that changes. When nobody is listening yet, the first listener
  /// also receives the current state once it is known.
  Stream<bool> observeEnabled() => BluetoothPlatform.instance.observeBluetoothEnabled();

  /// **Android only**
  /// Requests to enable Bluetooth on the device.
  /// Displays a system dialog asking the user to turn on Bluetooth.
  ///
  /// Note: This requires the `BLUETOOTH_CONNECT` permission to be granted beforehand.
  /// If the permission is not granted, this call will cause the app to crash.
  /// Permission handling is not managed by this package.
  Future<void> enable() => BluetoothPlatform.instance.enableBluetooth();
}
