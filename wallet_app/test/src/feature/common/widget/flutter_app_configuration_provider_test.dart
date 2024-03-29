import 'package:flutter/cupertino.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/configuration/flutter_app_configuration.dart';
import 'package:wallet/src/feature/common/widget/flutter_app_configuration_provider.dart';

void main() {
  const defaultMockConfig = FlutterAppConfiguration(
    idleLockTimeout: Duration(seconds: 10),
    backgroundLockTimeout: Duration(seconds: 20),
    version: 0,
  );

  group('hashCode', () {
    test('hashCode matches', () {
      const sameAsDefaultConfig = FlutterAppConfiguration(
        idleLockTimeout: Duration(seconds: 10),
        backgroundLockTimeout: Duration(seconds: 20),
        version: 0,
      );

      expect(defaultMockConfig.hashCode, sameAsDefaultConfig.hashCode);
    });

    test('hashCode !matches', () {
      const otherIdle = FlutterAppConfiguration(
        idleLockTimeout: Duration(seconds: 1337),
        backgroundLockTimeout: Duration(seconds: 20),
        version: 0,
      );
      const otherBackground = FlutterAppConfiguration(
        idleLockTimeout: Duration(seconds: 10),
        backgroundLockTimeout: Duration(seconds: 1337),
        version: 0,
      );
      const otherVersion = FlutterAppConfiguration(
        idleLockTimeout: Duration(seconds: 10),
        backgroundLockTimeout: Duration(seconds: 20),
        version: 1,
      );

      expect(defaultMockConfig.hashCode == otherIdle.hashCode, isFalse);
      expect(defaultMockConfig.hashCode == otherBackground.hashCode, isFalse);
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
      idleLockTimeout: Duration(seconds: 8),
      backgroundLockTimeout: Duration(seconds: 5),
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
