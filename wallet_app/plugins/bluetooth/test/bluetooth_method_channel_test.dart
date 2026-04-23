import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:bluetooth/bluetooth_method_channel.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  MethodChannelBluetooth platform = MethodChannelBluetooth();
  MethodChannel channel = platform.methodChannel;

  setUp(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(channel, (
      MethodCall methodCall,
    ) async {
      switch (methodCall.method) {
        case 'isBluetoothEnabled':
          return true;
        case 'enableBluetooth':
          return null;
        default:
          throw UnimplementedError('Method not supported: ${methodCall.method}');
      }
    });
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(channel, null);
  });

  test('isBluetoothEnabled', () async {
    expect(await platform.isBluetoothEnabled(), true);
  });

  test('enableBluetooth', () async {
    // Expect it completes without throwing
    await platform.enableBluetooth();
  });
}
