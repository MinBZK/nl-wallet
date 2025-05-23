import 'package:flutter/cupertino.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/configuration/flutter_app_configuration.dart';
import 'package:wallet/src/feature/common/widget/flutter_app_configuration_provider.dart';

void main() {
  const defaultMockConfig = FlutterAppConfiguration(
    idleWarningTimeout: Duration(seconds: 7),
    idleLockTimeout: Duration(seconds: 10),
    backgroundLockTimeout: Duration(seconds: 20),
    staticAssetsBaseUrl: 'https://example.com/',
    version: 0,
  );

  group('hashCode', () {
    test('hashCode matches', () {
      const sameAsDefaultConfig = FlutterAppConfiguration(
        idleWarningTimeout: Duration(seconds: 7),
        idleLockTimeout: Duration(seconds: 10),
        backgroundLockTimeout: Duration(seconds: 20),
        staticAssetsBaseUrl: 'https://example.com/',
        version: 0,
      );

      expect(defaultMockConfig.hashCode, sameAsDefaultConfig.hashCode);
    });

    test('hashCode !matches', () {
      final otherWarning = FlutterAppConfiguration(
        idleWarningTimeout: Duration(hours: 1337),
        idleLockTimeout: defaultMockConfig.idleLockTimeout,
        backgroundLockTimeout: defaultMockConfig.backgroundLockTimeout,
        staticAssetsBaseUrl: defaultMockConfig.staticAssetsBaseUrl,
        version: defaultMockConfig.version,
      );
      final otherIdle = FlutterAppConfiguration(
        idleWarningTimeout: defaultMockConfig.idleWarningTimeout,
        idleLockTimeout: Duration(hours: 1337),
        backgroundLockTimeout: defaultMockConfig.backgroundLockTimeout,
        staticAssetsBaseUrl: defaultMockConfig.staticAssetsBaseUrl,
        version: defaultMockConfig.version,
      );
      final otherBackground = FlutterAppConfiguration(
        idleWarningTimeout: defaultMockConfig.idleWarningTimeout,
        idleLockTimeout: defaultMockConfig.idleLockTimeout,
        backgroundLockTimeout: Duration(hours: 1337),
        staticAssetsBaseUrl: defaultMockConfig.staticAssetsBaseUrl,
        version: defaultMockConfig.version,
      );
      final otherStaticAssetsBaseUrlPrefix = FlutterAppConfiguration(
        idleWarningTimeout: defaultMockConfig.idleWarningTimeout,
        idleLockTimeout: defaultMockConfig.idleLockTimeout,
        backgroundLockTimeout: defaultMockConfig.backgroundLockTimeout,
        staticAssetsBaseUrl: 'https://other.example.com/',
        version: defaultMockConfig.version,
      );
      final otherVersion = FlutterAppConfiguration(
        idleWarningTimeout: defaultMockConfig.idleWarningTimeout,
        idleLockTimeout: defaultMockConfig.idleLockTimeout,
        backgroundLockTimeout: defaultMockConfig.backgroundLockTimeout,
        staticAssetsBaseUrl: defaultMockConfig.staticAssetsBaseUrl,
        version: 1337,
      );

      expect(defaultMockConfig.hashCode == otherWarning.hashCode, isFalse);
      expect(defaultMockConfig.hashCode == otherIdle.hashCode, isFalse);
      expect(defaultMockConfig.hashCode == otherBackground.hashCode, isFalse);
      expect(defaultMockConfig.hashCode == otherStaticAssetsBaseUrlPrefix.hashCode, isFalse);
      expect(defaultMockConfig.hashCode == otherVersion.hashCode, isFalse);
    });
  });

  testWidgets(
    'verify builder is called when a config is available',
    (tester) async {
      bool called = false;
      await tester.pumpWidget(
        FlutterAppConfigurationProvider(
          configProvider: const Stream.empty(),
          defaultConfig: defaultMockConfig,
          builder: (config) {
            called = true;
            return const SizedBox.shrink();
          },
        ),
      );

      expect(called, true);
    },
  );

  testWidgets('verify builder is called when a config is available', (tester) async {
    const expectedConfig = FlutterAppConfiguration(
      idleWarningTimeout: Duration(seconds: 7),
      idleLockTimeout: Duration(seconds: 8),
      backgroundLockTimeout: Duration(seconds: 5),
      staticAssetsBaseUrl: 'https://example.com/',
      version: 0,
    );
    late FlutterAppConfiguration receivedConfig;
    await tester.pumpWidget(
      FlutterAppConfigurationProvider(
        configProvider: Stream.value(expectedConfig),
        defaultConfig: defaultMockConfig,
        builder: (config) {
          receivedConfig = config;
          return const SizedBox.shrink();
        },
      ),
    );

    // Make sure stream is processed
    await tester.pumpAndSettle();

    expect(receivedConfig, expectedConfig);
  });
}
