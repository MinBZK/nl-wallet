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
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockStreamHandler(platform.stateChannel, null);
  });

  test('isBluetoothEnabled', () async {
    expect(await platform.isBluetoothEnabled(), true);
  });

  test('observeBluetoothEnabled maps state events to booleans', () async {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockStreamHandler(
      platform.stateChannel,
      MockStreamHandler.inline(
        onListen: (arguments, events) {
          events.success(false);
          events.success(true);
          events.endOfStream();
        },
      ),
    );

    expect(platform.observeBluetoothEnabled(), emitsInOrder([false, true, emitsDone]));
  });

  test('observeBluetoothEnabled keeps emitting to a listener after another listener cancels', () async {
    // Own instance, because the previous test ended the shared stream of `platform`
    final platform = MethodChannelBluetooth();
    var platformListenCount = 0;
    late MockStreamHandlerEventSink events;
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockStreamHandler(
      platform.stateChannel,
      MockStreamHandler.inline(
        onListen: (arguments, sink) {
          platformListenCount++;
          events = sink;
        },
      ),
    );

    final first = platform.observeBluetoothEnabled().listen((_) {});
    final received = <bool>[];
    final second = platform.observeBluetoothEnabled().listen(received.add);
    await pumpEventQueue();
    await first.cancel();
    events.success(true);
    await pumpEventQueue();

    expect(platformListenCount, 1);
    expect(received, [true]);
    await second.cancel();
  });

  test('enableBluetooth', () async {
    // Expect it completes without throwing
    await platform.enableBluetooth();
  });
}
