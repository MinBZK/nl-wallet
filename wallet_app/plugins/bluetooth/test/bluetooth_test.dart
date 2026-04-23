import 'package:bluetooth/bluetooth.dart';
import 'package:bluetooth/bluetooth_method_channel.dart';
import 'package:bluetooth/bluetooth_platform_interface.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:plugin_platform_interface/plugin_platform_interface.dart';

class MockBluetoothPlatform with MockPlatformInterfaceMixin implements BluetoothPlatform {
  @override
  Future<bool> isBluetoothEnabled() => Future.value(true);

  @override
  Future<void> enableBluetooth() async {}
}

void main() {
  final BluetoothPlatform initialPlatform = BluetoothPlatform.instance;

  test('$MethodChannelBluetooth is the default instance', () {
    expect(initialPlatform, isInstanceOf<MethodChannelBluetooth>());
  });

  test('isBluetoothEnabled', () async {
    Bluetooth bluetoothPlugin = Bluetooth();
    MockBluetoothPlatform fakePlatform = MockBluetoothPlatform();
    BluetoothPlatform.instance = fakePlatform;

    expect(await bluetoothPlugin.isEnabled(), true);
  });

  test('enableBluetooth', () async {
    Bluetooth bluetoothPlugin = Bluetooth();
    MockBluetoothPlatform fakePlatform = MockBluetoothPlatform();
    BluetoothPlatform.instance = fakePlatform;

    // Expect it completes normally
    await bluetoothPlugin.isEnabled();
  });
}
