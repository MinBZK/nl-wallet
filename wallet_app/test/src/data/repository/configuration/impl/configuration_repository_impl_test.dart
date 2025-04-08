import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/configuration/impl/configuration_repository_impl.dart';
import 'package:wallet/src/domain/model/configuration/flutter_app_configuration.dart';
import 'package:wallet_core/core.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late ConfigurationRepositoryImpl configurationRepository;
  late MockTypedWalletCore mockCore;

  setUp(() {
    mockCore = MockTypedWalletCore();
    configurationRepository = ConfigurationRepositoryImpl(mockCore);
  });

  test('verify that CoreConfigurationRepository fetches configuration through wallet_core', () async {
    when(mockCore.observeConfig()).thenAnswer(
      (_) => Stream.value(
        FlutterConfiguration(
          inactiveWarningTimeout: 3,
          inactiveLockTimeout: 5,
          backgroundLockTimeout: 10,
          version: BigInt.zero,
        ),
      ),
    );

    final config = await configurationRepository.appConfiguration.first;
    expect(
      config,
      const FlutterAppConfiguration(
        idleWarningTimeout: Duration(seconds: 3),
        idleLockTimeout: Duration(seconds: 5),
        backgroundLockTimeout: Duration(seconds: 10),
        version: 0,
      ),
    );
  });
}
