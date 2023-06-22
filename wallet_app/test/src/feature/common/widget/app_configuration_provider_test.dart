import 'package:flutter/cupertino.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/configuration/app_configuration.dart';
import 'package:wallet/src/feature/common/widget/app_configuration_provider.dart';

void main() {
  const defaultMockConfig = AppConfiguration(
    idleLockTimeout: Duration(seconds: 10),
    backgoundLockTimeout: Duration(seconds: 20),
  );

  testWidgets(
    'verify builder is called when a config is available',
        (tester) async {
      bool called = false;
      await tester.pumpWidget(
        AppConfigurationProvider(
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
    const expectedConfig = AppConfiguration(
      idleLockTimeout: Duration(seconds: 8),
      backgoundLockTimeout: Duration(seconds: 5),
    );
    late AppConfiguration receivedConfig;
    await tester.pumpWidget(
      AppConfigurationProvider(
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
