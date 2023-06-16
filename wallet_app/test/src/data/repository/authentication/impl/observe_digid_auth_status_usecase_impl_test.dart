import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/bridge_generated.dart';
import 'package:wallet/src/data/repository/authentication/digid_auth_repository.dart';
import 'package:wallet/src/data/repository/authentication/impl/digid_auth_repository_impl.dart';
import 'package:wallet/src/wallet_core/typed_wallet_core.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late DigidAuthRepository authRepository;
  late TypedWalletCore core = Mocks.create();

  setUp(() {
    authRepository = DigidAuthRepositoryImpl(core);
  });

  group('DigiD Auth Url', () {
    test('auth url should be fetched through the wallet_core', () async {
      expect(await authRepository.getAuthUrl(), kMockDigidAuthUrl);
      verify(core.getDigidAuthUrl());
    });
  });

  group('DigiD State Updates', () {
    test('Stream updates when notified', () async {
      expectLater(authRepository.observeAuthStatus(), emitsInOrder([DigidAuthStatus.authenticating]));
      authRepository.notifyDigidStateUpdate(DigidState.Authenticating);
    });

    test('Stream reflects success state and returns back to idle', () async {
      expectLater(authRepository.observeAuthStatus(),
          emitsInOrder([DigidAuthStatus.authenticating, DigidAuthStatus.success, DigidAuthStatus.idle]));
      authRepository.notifyDigidStateUpdate(DigidState.Authenticating);
      authRepository.notifyDigidStateUpdate(DigidState.Success);
    });

    test('Stream reflects error state and returns back to idle', () async {
      expectLater(authRepository.observeAuthStatus(),
          emitsInOrder([DigidAuthStatus.authenticating, DigidAuthStatus.error, DigidAuthStatus.idle]));
      authRepository.notifyDigidStateUpdate(DigidState.Authenticating);
      authRepository.notifyDigidStateUpdate(DigidState.Error);
    });

    test('Stream returns back to idle when receiving a null status', () async {
      expectLater(authRepository.observeAuthStatus(), emitsInOrder([DigidAuthStatus.idle]));
      authRepository.notifyDigidStateUpdate(null);
    });
  });
}
