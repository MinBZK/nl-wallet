import 'package:flutter/cupertino.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/bridge_generated.dart';
import 'package:wallet/src/feature/common/widget/flutter_configuration_provider.dart';

void main() {
  const defaultMockConfig = FlutterConfiguration(inactiveLockTimeout: 5, backgroundLockTimeout: 10);

  testWidgets(
    'verify builder is called when a config is available',
    (tester) async {
      bool called = false;
      await tester.pumpWidget(
        FlutterConfigurationProvider(
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
    const expectedConfig = FlutterConfiguration(inactiveLockTimeout: 8, backgroundLockTimeout: 9);
    late FlutterConfiguration receivedConfig;
    await tester.pumpWidget(
      FlutterConfigurationProvider(
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

    expect(receivedConfig.backgroundLockTimeout, expectedConfig.backgroundLockTimeout);
    expect(receivedConfig.inactiveLockTimeout, expectedConfig.inactiveLockTimeout);
  });
}
