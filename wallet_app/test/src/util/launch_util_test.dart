import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/util/launch_util.dart';

const kUrlLauncherMethodChannel = MethodChannel('plugins.flutter.io/url_launcher');

void main() {
  setUp(() async {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
      kUrlLauncherMethodChannel,
      (MethodCall methodCall) async {
        final url = (methodCall.arguments as Map<Object?, Object?>)['url'];
        if (url == 'https://example.org/valid') return true;
        throw UnsupportedError('url not supported: $url');
      },
    );
  });

  tearDown(() async {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
      kUrlLauncherMethodChannel,
      null,
    );
  });

  test('launchUrlStringCatching does not throw exception with invalid url', () async {
    final result = await launchUrlStringCatching('::Not valid URI::');
    expect(result, isFalse);
  });

  test('launchUrlStringCatching returns true for valid uri', () async {
    final result = await launchUrlStringCatching('https://example.org/valid');
    expect(result, isTrue);
  });

  test('launchUriCatching returns true for valid uri', () async {
    final result = await launchUriCatching(Uri.parse('https://example.org/valid'));
    expect(result, isTrue);
  });
}
