import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/bridge_generated.dart';
import 'package:wallet/src/data/repository/pid/impl/pid_repository_impl.dart';
import 'package:wallet/src/data/repository/pid/pid_repository.dart';
import 'package:wallet/src/wallet_core/typed_wallet_core.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late PidRepository authRepository;
  late TypedWalletCore core = Mocks.create();

  setUp(() {
    authRepository = PidRepositoryImpl(core);
  });

  group('DigiD Auth Url', () {
    test('auth url should be fetched through the wallet_core', () async {
      expect(await authRepository.getPidIssuanceUrl(), kMockPidIssuanceUrl);
      verify(core.createPidIssuanceRedirectUri());
    });
  });

  group('DigiD State Updates', () {
    test('Stream updates when notified', () async {
      expectLater(authRepository.observePidIssuanceStatus(), emitsInOrder([PidIssuanceStatus.authenticating]));
      authRepository.notifyPidIssuanceStateUpdate(const PidIssuanceEvent.authenticating());
    });

    test('Stream reflects success state and returns back to idle', () async {
      expectLater(authRepository.observePidIssuanceStatus(),
          emitsInOrder([PidIssuanceStatus.authenticating, PidIssuanceStatus.success, PidIssuanceStatus.idle]));
      authRepository.notifyPidIssuanceStateUpdate(const PidIssuanceEvent.authenticating());
      authRepository.notifyPidIssuanceStateUpdate(PidIssuanceEvent.success(previewCards: List.empty()));
    });

    test('Stream reflects error state and returns back to idle', () async {
      expectLater(authRepository.observePidIssuanceStatus(),
          emitsInOrder([PidIssuanceStatus.authenticating, PidIssuanceStatus.error, PidIssuanceStatus.idle]));
      authRepository.notifyPidIssuanceStateUpdate(const PidIssuanceEvent.authenticating());
      authRepository.notifyPidIssuanceStateUpdate(const PidIssuanceEvent.error(data: 'data'));
    });

    test('Stream returns back to idle when receiving a null status', () async {
      expectLater(authRepository.observePidIssuanceStatus(), emitsInOrder([PidIssuanceStatus.idle]));
      authRepository.notifyPidIssuanceStateUpdate(null);
    });
  });
}
