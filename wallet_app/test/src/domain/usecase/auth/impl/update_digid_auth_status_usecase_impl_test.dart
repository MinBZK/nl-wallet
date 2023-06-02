import 'package:core_domain/core_domain.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/usecase/auth/impl/update_digid_auth_status_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/auth/update_digid_auth_status_usecase.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late UpdateDigidAuthStatusUseCase usecase;
  final mockRepo = Mocks.create<MockDigidAuthRepository>();

  setUp(() {
    usecase = UpdateDigidAuthStatusUseCaseImpl(mockRepo);
  });

  group('DigiD Status Updates', () {
    test('DigiD state should be passed on to the repository', () async {
      usecase.invoke(DigidState.success);
      verify(mockRepo.notifyDigidStateUpdate(DigidState.success));
    });
  });
}
