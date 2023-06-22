import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/bridge_generated.dart';
import 'package:wallet/src/data/repository/configuration/core/core_configuration_repository.dart';
import 'package:wallet/src/data/repository/configuration/mock/mock_configuration_repository.dart';
import 'package:wallet/src/domain/model/configuration/app_configuration.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late CoreConfigurationRepository configurationRepository;
  late MockTypedWalletCore mockCore;

  setUp(() {
    mockCore = MockTypedWalletCore();
    configurationRepository = CoreConfigurationRepository(mockCore);
  });

  test('verify that CoreConfigurationRepository fetches configuration through wallet_core', () async {
    when(mockCore.observeConfig()).thenAnswer(
      (_) => Stream.value(
        const FlutterConfiguration(inactiveLockTimeout: 5, backgroundLockTimeout: 10),
      ),
    );

    final config = await configurationRepository.appConfiguration.first;
    expect(
      config,
      const AppConfiguration(
        idleLockTimeout: Duration(seconds: 5),
        backgoundLockTimeout: Duration(seconds: 10),
      ),
    );
  });

  test('verify that MockConfigurationRepository provides a configuration', () async {
    final config = await MockConfigurationRepository().appConfiguration.first;
    expect(config, isNotNull);
  });
}
