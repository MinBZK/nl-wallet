import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/configuration/impl/configuration_repository_impl.dart';
import 'package:wallet_core/core.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.dart';

void main() {
  late MockTypedWalletCore mockCore;

  late ConfigurationRepositoryImpl configurationRepository;

  setUp(() {
    mockCore = MockTypedWalletCore();

    configurationRepository = ConfigurationRepositoryImpl(
      mockCore,
      MockMapper(),
    );
  });

  test('verify that ConfigurationRepository fetches configuration through wallet_core', () async {
    when(mockCore.observeConfig()).thenAnswer(
      (_) => Stream.value(
        const FlutterConfiguration(
          inactiveWarningTimeout: 3,
          inactiveLockTimeout: 5,
          backgroundLockTimeout: 10,
          pidAttestationTypes: ['urn:eudi:pid:nl:1'],
          staticAssetsBaseUrl: 'https://example.com/',
          version: '0',
          environment: 'test',
        ),
      ),
    );

    final config = await configurationRepository.observeAppConfiguration.first;
    expect(
      config,
      WalletMockData.flutterAppConfiguration,
    );
  });
}
